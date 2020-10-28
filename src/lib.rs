//
// src
// lib.rs: Implements the memory map of the virtual hardware that OwOS runs on.
//
// Created by jenra.
// Created on October 24 2020.
//

use std::collections::VecDeque;
use std::fs;
use std::io::Error;

use emulator_6502::Interface6502;
use emulator_6502::MOS6502;

// Memory Map:
// The 6551 is used for uart communication
//
// Bank 0 specific addresses:
// $0000       - bank select
// $f000-$ffff - boot disc
//
// Bank 1:
// $0000-$00fb - zero page
// $00f7       - disc data
// $00f8       - disc select
// $00f9       - disc address low
// $00fa       - disc page low
// $00fb       - disc page high
// $00fc       - uart data in/out
// $00fd       - uart status register/programmed reset
// $00fe       - uart command register
// $00ff       - uart control register
// $0100-$01ff - stack
pub struct MemoryMap {
	// UART
	uart_data_tx: u8,
	uart_data_rx: u8,
	uart_data_buf: VecDeque<u8>,
	uart_stat: u8,
	uart_cmmd: u8,
	uart_ctrl: u8,

	// Discs
	disc_0: Vec<u8>,
	disc_1: Vec<u8>,
	disc_select: u8,
	disc_addr: u8,
	disc_page: u16,

	// Memory
	bank_0: u8,
	ram: Box<[u8; 1 << 20]>,
	rom: Vec<u8>
}

impl MemoryMap {
	// Creates a new memory map
	pub fn new(rom_file: &str) -> Result<MemoryMap, Error> {
		let rom = fs::read(rom_file)?;
		Ok(MemoryMap {
			uart_data_tx: 0b0000_0000,
			uart_data_rx: 0b0000_0000,
			uart_data_buf: VecDeque::new(),
			uart_stat: 0b0001_0000,
			uart_cmmd: 0b0000_0010,
			uart_ctrl: 0b0000_0000,

			disc_0: Vec::with_capacity(1 << 24),
			disc_1: Vec::with_capacity(1 << 24),

			disc_select: 0,
			disc_addr: 0,
			disc_page: 0,

			bank_0: 0,
			ram: Box::new([0; 1 << 20]),
			rom
		})
	}

	// Resets the memory map
	pub fn reset(&mut self) {
		self.uart_data_tx  = 0b0000_0000;
		self.uart_data_rx  = 0b0000_0000;
		self.uart_data_buf.clear();
		self.uart_stat    &= 0b0110_0000;
		self.uart_stat    |= 0b0001_0000;
		self.uart_cmmd     = 0b0000_0010;
		self.uart_ctrl     = 0b0000_0000;

		self.bank_0 = 0;
	}

	// Receives a piece of data from UART and puts it in the buffer
	pub fn receive_uart_data(&mut self, cpu: &mut MOS6502, data: u8) {
		// Check if enabled
		if self.uart_cmmd & 0b0000_0001 == 0 {
			return;
		}

		// Save data
		if self.uart_data_rx == 0 {
			self.uart_data_rx = data;
		} else {
			self.uart_data_buf.push_back(data);
		}

		// Echo if enabled
		if self.uart_cmmd & 0b0001_0000 != 0 {
			ncurses::putp(&format!("{}", data as char));

			if data as char == '\n' {
				ncurses::putp("\r");
			}

			ncurses::refresh();
		}

		// Request interrupt if enabled
		if self.uart_cmmd & 0b0000_0010 == 0 {
			cpu.interrupt_request();
			self.uart_stat |= 0b1000_0000;
		} else {
			self.uart_stat &= 0b0111_1111;
		}

		// Set flags
		self.uart_stat |= 0b0000_1000;
	}
}

impl Interface6502 for MemoryMap {
	fn read(&mut self, addr: u16) -> u8 {
		// ROM
		#[allow(unused_comparisons)]
		if self.bank_0 == 0x00 && 0xf000 <= addr && addr <= 0xffff {
			self.rom[addr as usize - 0xf000]

		// Bank 0 access
		} else if self.bank_0 == 0x00 && addr == 0x0000 {
			self.bank_0
		} else {
			match addr {
				// Disc data
				0x00f7 => {
					let addr = self.disc_addr as usize | ((self.disc_page as usize) << 8);
					if self.disc_select == 0 {
						self.disc_0[addr]
					} else if self.disc_select == 1 {
						self.disc_1[addr]
					} else {
						0
					}
				}

				// Disc select and address
				0x00f8 => self.disc_select,
				0x00f9 => self.disc_addr,
				0x00fa => self.disc_page as u8,
				0x00fb => (self.disc_page >> 8) as u8,

				// UART
				0x00fc => {
					let data = self.uart_data_rx;

					if self.uart_data_buf.len() > 0 {
						self.uart_data_rx = self.uart_data_buf.pop_front().unwrap();
					} else {
						self.uart_data_rx = 0;
						self.uart_stat &= 0b1111_0111;
					}

					self.uart_stat &= 0b0111_1000;

					data
				}
				0x00fd => self.uart_stat,
				0x00fe => self.uart_cmmd,
				0x00ff => self.uart_ctrl,

				// RAM
				_      => self.ram[addr as usize]
			}
		}
	}

	fn write(&mut self, addr: u16, data: u8) {
		// ROM
		#[allow(unused_comparisons)]
		if self.bank_0 == 0x00 && 0xf000 <= addr && addr <= 0xffff {

		// Bank 0 access
		} else if self.bank_0 == 0x00 && addr == 0x0000 {
			self.bank_0 = data;
		} else {
			match addr {
				// Disc data
				0x00f7 => {
					let addr = self.disc_addr as usize | ((self.disc_page as usize) << 8);
					if self.disc_select == 0 {
						self.disc_0[addr] = data
					} else if self.disc_select == 1 {
						self.disc_1[addr] = data
					}
				}

				// Disc select and address
				0x00f8 => self.disc_select = data,
				0x00f9 => self.disc_addr = data,
				0x00fa => self.disc_page = (self.disc_page & 0xff00) | data as u16,
				0x00fb => self.disc_page = (self.disc_page & 0x00ff) | (data as u16) << 8,

				// UART
				0x00fc => {
					self.uart_data_tx = data;

					// Print character if enabled
					if self.uart_cmmd & 0b0000_0001 != 0 && data != 0 {
						ncurses::putp(&format!("{}", data as char));

						if data as char == '\n' {
							ncurses::putp("\r");
						}

						ncurses::refresh();

					// Otherwise set tx as not empty
					} else {
						self.uart_stat &= 0b1110_1111;
					}
				}
				0x00fd => {
					self.uart_cmmd &= 0b1110_0000;
					self.uart_cmmd |= 0b0000_0010;
					self.uart_stat &= 0b1111_1011;
				}
				0x00fe => {
					self.uart_cmmd = data;

					// Print character if enabled and tx is not empty
					if data & 0b0000_0001 != 0 && self.uart_stat & 0b0001_0000 == 0 {
						ncurses::putp(&format!("{}", self.uart_data_tx as char));

						if data as char == '\n' {
							ncurses::putp("\r");
						}

						ncurses::refresh();

						self.uart_stat |= 0b0001_0000;
					}
				}
				0x00ff => self.uart_ctrl = data,

				// RAM
				_      => self.ram[addr as usize] = data
			}
		}
	}
}

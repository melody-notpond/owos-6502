use std::sync::mpsc;
use std::thread;

use emulator_6502::MOS6502;

use owos_6502::MemoryMap;

fn main() {
	// Init screen
	ncurses::initscr();
	ncurses::noecho();
	ncurses::refresh();

	// Init cpu
	let mut cpu = MOS6502::new();
	let mut interface = MemoryMap::new("build-6502/bootrom.disc").expect("ROM image not found");
	cpu.reset(&mut interface);

	// Init getch thread
	let (tx, rx) = mpsc::channel();
	thread::spawn(move || {
		loop {
			let c = ncurses::getch();
			tx.send(c as u8).unwrap();
		}
	});

	// Run cpu
	loop {
		// Get data
		match rx.try_recv() {
			Ok(v) => interface.receive_uart_data(&mut cpu, v),
			Err(_) => ()
		}

		// Execute instruction
		cpu.execute_instruction(&mut interface);
	}
}

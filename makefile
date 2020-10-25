ASM = asm6502
BOOTROM_ARGS = -d -s F000 -o build-6502/bootrom.disc

all: bootrom

bootrom: src-6502/boot/bootrom.s
	-mkdir build-6502
	$(ASM) $(BOOTROM_ARGS) $?

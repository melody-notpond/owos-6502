ASM = asm6502
BOOTROM_ARGS = -d -s F000 -o build-6502/bootrom.disc

all: bootrom

mkdirs:
	-mkdir build-6502

bootrom: src-6502/boot/*
	$(ASM) $(BOOTROM_ARGS) $?

.origin $f000
_start:
	; Set control register
	; S = stop bit count (0 = 1 stop bit)
	; WW = word size (01 = 7 bits)
	; C = clock generator source (1 = baud rate generator)
	; BBBB = baudrate (1110 = 9600 baud)
	;     SWWCBBBB
	lda #%00111110
	sta $FF

	; PPP = parity settings (000 = no parity)
	; E = echo mode enable/disable (0 = echo mode disabled)
	; TT = transmitter controls (no idea what this does but it's set to 00)
	; I = receiver interrupt enable (1 = disable interrupt)
	; R = data terminal ready (1 = enable receiver/transmitter)
	;     PPPETTIR
	lda #%00000011
	sta $FE

	; Print the message
	ldx #<options
	ldy #>options
	jsr puts

	; Initial state is option 0
	lda #$00
	sta $04

; Get the key
get_key:
	jsr getc

	; No key = no action
	beq get_key

	; ANSI escape character
	cmp #$1B
	beq move_arrow

	; Enter (newline)
	cmp #$0A
	bne get_key

; Loop forever
loop:
	jmp loop

; Check if the inputted ANSI code is a vertical arrow key
move_arrow:
	; The bracket in the ANSI escape code
	jsr getc
	cmp #$5B
	bne get_key
	jsr getc

	; Up key
	cmp #$41
	beq up_key

	; Down key
	cmp #$42
	beq down_key

	; Not a valid key
	jmp get_key

up_key:
	; Don't move if at the 0th option
	lda $04
	beq get_key

	; Update state
	dec $04

	; Move cursor
	lda #$1b
	jsr putc
	lda #$5B
	jsr putc
	lda #$01
	jsr putc
	lda #$41
	jsr putc
	jmp get_key

down_key:
	; Don't move if at the 2nd option
	lda $04
	cmp #$02
	beq get_key

	; Update state
	inc $04

	; Move cursor
	lda #$1b
	jsr putc
	lda #$5B
	jsr putc
	lda #$01
	jsr putc
	lda #$42
	jsr putc
	jmp get_key


; putc(c: u8) -> void
; Prints a single character onto the UART port. The A
; register contains the character being printed. (Note:
; although this is wasteful on the emulator, in the future
; I plan to build a computer that can run this OS, which
; means putc will have to have timing stuff, so this function
; serves as a reminder and to ease refactoring in the future.)
putc:
	sta $FC
	rts


; getc() -> u8
; Waits until a character is available and returns it via the A
; register.
getc:
	; Test if the UART receiver is full
	lda $FD
	and #%00001000
	beq getc

	; Get the character
	lda $FC
	rts


; puts(str: u16) -> void
; Prints a string onto the UART port. The low byte of the
; string pointer is passed through the X register, and
; the high byte is passed through the Y register.
puts:
	; Save the string in $01
	stx $01
	sty $02
	ldy #$00

puts_loop:
	; Get the character and return if it's null
	lda ($01), Y
	beq puts_rts

	; Put the character and move onto the next character
	jsr putc
	iny
	jmp puts_loop

puts_rts:
	rts


; The options menu shown on boot
options:
	.bytes "owOS Boot shell"         , $0A,
	.bytes "Please select an option:", $0A,
	.bytes "  Boot from disc 0"      , $0A,
	.bytes "  Boot from disc 1"      , $0A,
	.bytes "  Recovery shell"        ,
	.bytes $1B, "[", $01, "F", $1B, "[", $01, "F", $00

.origin $fffc
.word _start

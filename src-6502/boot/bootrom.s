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
	ldx #<message
	ldy #>message
	jsr puts

; loop forever
loop:
	lda $FC
	sta $FC
	jmp loop

; puts(str: u16) -> void
; Prints a string onto the UART port. The low byte of the
; string pointer is passed through the X register, and
; the high byte is passed through the Y register.
puts:
	stx $01
	sty $02
	ldy #$00
puts_loop:
	lda ($01), Y
	beq puts_rts
	sta $FC
	iny
	jmp puts_loop
puts_rts:
	rts

message:
	.bytes "Hewwo worwd! uwu", $0A, $00

.origin $fffc
.word _start

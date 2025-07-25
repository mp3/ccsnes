; Simple SNES Hello World Test ROM
; This creates a minimal ROM that displays "HELLO" on screen

.include "snes.inc"

.segment "HEADER"
    .byte "HELLO WORLD TEST     " ; Title (21 bytes)
    .byte $30                      ; LoROM, FastROM
    .byte $00                      ; No chips
    .byte $08                      ; ROM size (256KB)
    .byte $00                      ; No RAM
    .byte $00                      ; Japan
    .byte $33                      ; Nintendo
    .byte $00                      ; Version
    .word $0000                    ; Checksum complement
    .word $FFFF                    ; Checksum

.segment "VECTORS"
    .word 0, 0          ; Native mode interrupts (unused)
    .word 0, 0, 0
    .word 0, 0
    .word Reset         ; Reset vector

.segment "CODE"
Reset:
    sei                 ; Disable interrupts
    clc                 ; Clear carry
    xce                 ; Switch to native mode
    
    rep #$30           ; 16-bit A/X/Y
    
    ; Setup stack
    ldx #$1FFF
    txs
    
    ; Clear registers
    lda #$0000
    tcd                ; Direct page = 0
    
    ; Turn off screen
    sep #$20           ; 8-bit A
    lda #$8F
    sta $2100          ; Screen off
    
    ; Setup video mode
    lda #$09
    sta $2105          ; Mode 1, BG3 priority
    
    ; Setup BG1 tilemap
    lda #$00
    sta $2107          ; BG1 tilemap at VRAM $0000
    
    ; Setup BG1 tiles
    lda #$01
    sta $210B          ; BG1 tiles at VRAM $1000
    
    ; Clear VRAM
    rep #$20
    lda #$0000
    sta $2116          ; VRAM address
    ldx #$8000         ; Clear 32KB
ClearVRAM:
    stz $2118
    dex
    bne ClearVRAM
    
    ; Upload font tiles
    lda #$1000
    sta $2116          ; VRAM address for tiles
    ldx #$0000
UploadFont:
    lda FontData,x
    sta $2118
    inx
    inx
    cpx #$0100         ; 16 tiles * 8 bytes each
    bne UploadFont
    
    ; Write "HELLO" to tilemap
    lda #$0000
    sta $2116          ; VRAM address
    
    ; H
    lda #$0048
    sta $2118
    ; E
    lda #$0045
    sta $2118
    ; L
    lda #$004C
    sta $2118
    ; L
    lda #$004C
    sta $2118
    ; O
    lda #$004F
    sta $2118
    
    ; Turn on screen
    sep #$20
    lda #$0F
    sta $2100          ; Screen on, full brightness
    
    ; Enable NMI
    lda #$80
    sta $4200
    
MainLoop:
    wai                ; Wait for interrupt
    bra MainLoop

; Simple font data (partial ASCII)
FontData:
    ; Tile $45 - 'E'
    .byte $7E, $40, $40, $7C, $40, $40, $7E, $00
    .byte $00, $00, $00, $00, $00, $00, $00, $00
    
    ; Tile $48 - 'H'
    .byte $42, $42, $42, $7E, $42, $42, $42, $00
    .byte $00, $00, $00, $00, $00, $00, $00, $00
    
    ; Tile $4C - 'L'
    .byte $40, $40, $40, $40, $40, $40, $7E, $00
    .byte $00, $00, $00, $00, $00, $00, $00, $00
    
    ; Tile $4F - 'O'
    .byte $3C, $42, $42, $42, $42, $42, $3C, $00
    .byte $00, $00, $00, $00, $00, $00, $00, $00
.text
.org 0x0000

.set START, _start
.set INT, _int
.set HBLANK, _int
.set VBLANK, _int

.long 0x0             ; Initial Stack Address
.long START           ; Start of program Code
.long INT             ; Bus error
.long INT             ; Address error
.long INT             ; Illegal instruction
.long INT             ; Division by zero
.long INT             ; CHK exception
.long INT             ; TRAPV exception
.long INT             ; Privilage violation
.long INT             ; TRACE exception
.long INT             ; Line-A emulator
.long INT             ; Line-F emulator
.long INT             ; Reserved (NOT USED)
.long INT             ; Co-processor protocol violation
.long INT             ; Format error
.long INT             ; Uninitialized Interrupt
.long INT             ; Reserved (NOT USED)
.long INT             ; Reserved (NOT USED)
.long INT             ; Reserved (NOT USED)
.long INT             ; Reserved (NOT USED)
.long INT             ; Reserved (NOT USED)
.long INT             ; Reserved (NOT USED)
.long INT             ; Reserved (NOT USED)
.long INT             ; Reserved (NOT USED)
.long INT             ; Spurious Interrupt
.long INT             ; IRQ Level 1
.long INT             ; IRQ Level 2 (EXT Interrupt)
.long INT             ; IRQ Level 3
.long HBLANK          ; IRQ Level 4 (VDP Horizontal Interrupt)
.long INT             ; IRQ Level 5
.long VBLANK          ; IRQ Level 6 (VDP Vertical Interrupt)
.long INT             ; IRQ Level 7
.long INT             ; TRAP #00 Exception
.long INT             ; TRAP #01 Exception
.long INT             ; TRAP #02 Exception
.long INT             ; TRAP #03 Exception
.long INT             ; TRAP #04 Exception
.long INT             ; TRAP #05 Exception
.long INT             ; TRAP #06 Exception
.long INT             ; TRAP #07 Exception
.long INT             ; TRAP #08 Exception
.long INT             ; TRAP #09 Exception
.long INT             ; TRAP #10 Exception
.long INT             ; TRAP #11 Exception
.long INT             ; TRAP #12 Exception
.long INT             ; TRAP #13 Exception
.long INT             ; TRAP #14 Exception
.long INT             ; TRAP #15 Exception
.long INT             ; (FP) Branch or Set on Unordered Condition
.long INT             ; (FP) Inexact Result
.long INT             ; (FP) Divide by Zero
.long INT             ; (FP) Underflow
.long INT             ; (FP) Operand Error
.long INT             ; (FP) Overflow
.long INT             ; (FP) Signaling NAN
.long INT             ; (FP) Unimplemented Data Type
.long INT             ; MMU Configuration Error
.long INT             ; MMU Illegal Operation Error
.long INT             ; MMU Access Violation Error
.long INT             ; Reserved (NOT USED)
.long INT             ; Reserved (NOT USED)
.long INT             ; Reserved (NOT USED)
.long INT             ; Reserved (NOT USED)
.long INT             ; Reserved (NOT USED)

.ascii  "SEGA MEGADRIVE  "      ; SEGA must be the first four chars for TMSS
.ascii  "(C)2021.JAN     "
.ascii  "TESTPROJECT     "      ; export name
.ascii  "                "
.ascii  "                "
.ascii  "TESTPROJECT     "      ; domestic (Japanese) name
.ascii  "                "
.ascii  "                "
.ascii  "GM MK-0000 -01"
.short  0x0000                  ; checksum - not needed
.ascii  "J6              "
.long   0x00000000, 0x0007ffff  ; ROM start, end
.long   0x00ff0000, 0x00ffffff  ; RAM start, end
.ascii  "            "          ; no SRAM
.ascii  "    "
.ascii  "        "
.ascii  "        "              ; memo
.ascii  "                "
.ascii  "                "
.ascii  "F               "      ; enable any hardware configuration

.global _start
.global run_game
_start:
  bra #run_game

.global abort
abort:
  bra #run_game

_int:
  rte

;
; Missing compiler routines
;

.global __mulsi3
__mulsi3:
  bra #__mulsi3
;  move.l d2, -(sp)
;  move.l d0, -(sp)
;  mulu d1, d0
;  move.l d1, d2
;  mulu (sp)+, d1
;  swap d2
;  mulu (sp)+, d2
;  add.w d2, d1
;  swap d1
;  move.l (sp)+, d2
;  clr.w d1
;  add.l d1, d0
;  rts

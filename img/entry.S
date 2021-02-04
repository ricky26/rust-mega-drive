.global _start
_start:
  move.l #run_game, a0
  jmp (a0)

main:
    ldi 7 ; 0
    call f ; 2
    hlt             ; 4 | Expect A = 7.

f:
    push A          ; 6 Save this invocation's argument.
    call g          ; 8 g is allowed to overwrite A.
    pop A           ; 10 Recover our original argument.
    ret         ; 12 

g:
    ldi 123         ;14 Overwrite A.
    ret ; 16
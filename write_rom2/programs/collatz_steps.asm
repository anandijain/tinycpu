
init:
    steps = 1000
    ldi 0
    sta steps

    n = 1002
    ldi 6
    sta n

    one = 1004
    ldi 1
    sta one

    three = 1006
    ldi 3
    sta three


loop:
    ; if n == 1, jmp end
    lda n
    ldb one
    jeq end

    ; increment steps
    ldi 1
    add steps
    sta steps

    ; is_even(n)
    lda n
    and one ; this overwrites the A register with the result (similar to add and mul)
            ; note the value 1 will be in B register as a result of the and
    jz even

odd: 
    lda n
    mul three
    add one
    sta n
    jmp loop

even: 
    lda n
    shr
    sta n
    jmp loop
    

end: 
    lda steps
    hlt

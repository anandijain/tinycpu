; f(x) = x + 1 
;f(x, y) = x + y
x = 1000
ldi 5 
sta x

main: 
    lda x
    call f
    hlt

f: 
    

    ;pop x1 ; mov(ram[sp] -> x1)
    ;pop x2 
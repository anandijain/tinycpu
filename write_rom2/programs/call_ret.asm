x = 1000
y = 1002
ldi 0 ; 0 
sta x ; 2
sta y ; 4

main: 
    call f ; 6
    call f ; 8
    lda x ; 10
    ldb y ; 12
    ; expect 2 in A and 4 in B
    hlt ; 14

f: 
    ldi 1 ; 16
    add x ; 18
    sta x ; 20
    call g ; 22
    ret ; 24

g:
    ldi 2 ; 26
    add y ; 28
    sta y ; 30
    ret ; 32

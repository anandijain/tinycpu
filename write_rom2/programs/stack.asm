; (eq? 1 2)
; (if (eq? (+ 1 1) (+ 0 2)) 1 0)
; f(g(x))
; x = 1
; g() = 3x
; f() = g() + 3
def f():
    return g() + 3

def g(): 
    return 3x 

x = 1
f()


stack_init:
    stack_ptr_loc = 4094
    ldi 4092
    sta stack_ptr_loc ; never changes
    stack_ptr = 4092
    ldi 4090
    sta stack_ptr

init: 
    x = 1000
    ldi 1 
    sta x

    three = 1002
    ldi 3 
    sta three

; this is no good if f needs to be called multiple times because we would need to manage
; after_f1:, after_f2: etc 
main: 
    call f
after_f:
    hlt

f: 
    call g
after_g:
    add three
    ret

g:
    lda x
    mul three
    ret


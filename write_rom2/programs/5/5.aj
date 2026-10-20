(= a 1)
(= b (+ a 2))

a = 4094
LDI 1
STA a
b = 4092
LDI 2
ADD a

(= a 1)
(= b 2)
(= c 3)
(= d (+ (+ a b) (+ 1 c)))

a = 4094
ldi 1 
sta a 
b = 4092
ldi 2 
sta b
c = 4090
ldi 2 
sta c
compile(+ a b)
define_intermediate(%1)
sta %1
compile(+ 1 c)
add %1

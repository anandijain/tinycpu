(= steps 0)
(= n 5)
(if (eq? n 1))


(eq? a b)
(eq? 1 2)
(eq? a 1)
(eq? 1 a)

(if true 1 2) short to 1
(if false 1 2) => 2

(if (eq? 1 2) 1 2)

(eq? ex1 ex2)
the result of this is that when evaluated, the "true" ie just 1 gets stored in a, otherwise zero

compile(ex1, ctx)
sta next
compile(ex2)


(fn (square x) (* x x))


head => n args

+ => 2
= => 2
if => 3
eq? => 2
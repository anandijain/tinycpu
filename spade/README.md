# tinycpu

## notes

storing data: sequential logic

we start with an OR gate with inputs A and B, and output O

we tie B to O, so B == O always

now power on, a,b,o = 0
now turn A high so A(0->1)=> O(0->1) = B(0=>1)

now the output is permanently high so we have no way to reset the set output

the next extension is to have two nor gates

a,b -> nor1 -> o1
c,d -> nor2 -> o2

tie o1 = c, and o2 = b 
so now really only 2 inputs a and d 

we turn on the machine and which ever nor gate is faster turns out to 1, which holds the other out low
this is the sr latch
the problem is there are two inputs and if both are high we have invalid state in the sense that the SR outputs are usually called 
q and qbar and the notational implication is that q = !qbar but this is violated if both S and R are hi


there are two choices when deciding how to add a LOAD signal to the d flip flop, either you could gate the clock with an AND, but that could do weird things 
(imagine the load going high in the middle of a clock pulse, now we made a rising edge in the middle of the clock hi)

or we can do the one in ben eater video

# todo 
go back to tri state buffer/logic
still dont really understand what happens if 2 registers are enabled and writing to bus



# fetch 
pc_out, mar in 
mem_hi_we, 
mar_inc, 
ram_out, inst_in, pc_inc, 
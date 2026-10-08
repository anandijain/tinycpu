
adder
C A B | Ca Sum
000  | 0   0
001  | 0   1
010  | 0   1
011  | 1   0
100  | 0   1
101  | 1   0
110  | 1   0
111  | 1   1 

  1011
+ 0111
------

A1 + B1:
Ca: 1 Sum1 0

Ca1 + A2 + B2: Ca2: 1 Sum2: 1


LDA 14
ADD 15
OUT

LDA 14
LDB 15
how to explicitly read from ALU/put on the bus 



to fetch an instruction amounts to, placing the program counter onto the bus, and latching into the mem addr reg
then outputing the contents of ram pointed to by the mem_addr_reg onto bus and latching it into the instruction register
and also incrementing the program counter. 

FETCH
-----
pc_out, mar_in
ram_out, inst_in, pc_we/pc_inc



there are two ways to easily implement the control unit
the first important thing to realize is that when we manually control the control lines, we are keeping two pieces
of state in our minds: 1) is the opcode 2) is the step within the instruction

we can then build a truth table
(opcode, instr_step) -> control lines

there are two ways that we can easily implement this logic 1) gates 2) ROM

the easiest part to start with is instruction fetching, because it doesnt depend on the opcode,

so what we will do is 

lda 14
add 15


0001 1110
0010 1111

- rom input

clk0  0000 0000 -> xxxx 0000 -> pc_out, mar_in
clk1  0000 0001 -> xxxx 0001 -> ram_out, instr_in, pc_we
clk2  0001 0010 -> 0001 0010 -> mar_in (will put 4lsb instr into mar, so now has 14)
clk3  0001 0011 -> ram_out, a_in.
clk4  0001 0100 ... all zero controls until 0001 1111 (for now, will be slow )
clk15 0001 1111 at this point PC=0001
clk16 0001 0000 -> fetch
clk18 0010 0010 

opcodes

0001 LDA
0010 ADD - ldb

the order of control signals:
0 - A_WE
1 - B_WE
2 - C_WE
3- mar
4 - ram_we
5 - pc_inc/pc_we
6 - inst_we

789 - regsiter select mux
10 - reg manual select
11 - pc_clr 


the register select mux has the following values
0 - a
1- b
2-c
3-alu
4-ram
5-pc
6-reg_inst 

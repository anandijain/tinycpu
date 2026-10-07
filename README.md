
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



to fetch an instruction amounts to, placing the program counter onto the bus, and latching into the mem addr reg
then outputing the contents of ram pointed to by the mem_addr_reg onto bus and latching it into the instruction register
and also incrementing the program counter. 


pc_out, mar_in
ram_out, inst_in, pc_we/pc_inc

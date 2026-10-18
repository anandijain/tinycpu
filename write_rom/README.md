LDI 10 
STA 15
LDI 15
STA 14 
LDI 0
LDA 15
ADD 14 
HLT 0

AB | A XOR B
00 | 0
01 | 1
10 | 1
11 | 0


plan: 

* manually compute 1+2
* write the program that computes 1+2, store in ram, run program while manually doing the control lines 


LDI 1000
STA 4095
LDI 2000
ADD 4095

the fetch microinstructions when the instruction is 2 bytes on this computer will be

clk 1: pc_out, mar_in
clk 2: ram_out, inst_msb_in, pc_inc
clk 3: pc_out, mar_in
clk 4: ram_out, inst_lsb_in, pc_inc



STA is now more complicated because the 16 bit registers will need 2 bytes of ram.

lets say fetch is done and we want to store 1 << 8 | 1 in ram say at 4094 (0xffe)

then we start with
```
inst_out, mar_in
a_out, ram_msb_we, 
```
nd if registers were 8 bits that would be it,
but now we need to increment the MAR 

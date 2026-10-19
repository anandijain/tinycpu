at a high level this program needs to do two things

1) take an assembly program and write the contents of ram to a hex-words file
2) output the contents of ROM that is used to map the (opcode, inst_step, control_flags) -> control_lines used to implement the microinstructions/operations 



Lda 0x123  
inst_out, mar_in,
mem_hi_we,
mar_inc,
ram_out, a_in
inst_step_reset

Sta 0x123 A: 0xABCD
instr_out, mar_in
a_out, ram_msb_we
mar_inc, 
a_out, ram_lsb_we
inst_step_reset 



## compiler levels
### 1. 
In the first level we call it "symbolic assembly"
basically we have a way to tell the compiler at the start of the program certain addresses for variables. so x ADDR 4094 says that x's address in ram is 4094

then in normal assembly instructions, instead of providing an address manually, we can refer to symbols that then get looked up during compilation

### 2 
Now in 2.txt we abandon assembly and instead start assigning values to variables.
So as the compiler goes along, if it sees an assignment operation like `x=5` 
then it will look in a table to check if the identifier x already has a place in memory, if not it subtracts 2 bytes (or 1 word) from a counter and adds x to this table.
so the program then emits (symbolic?) assembly code.

in a case liek x=5
we write to the compilers Context{id: "x", addr: 4094}
then we emit the instructions 
LDI 5 
STA 4094

then when we go to the next line we see that y=3 and add to context and emit
LDI 3 STA 4092

now here we see c = a + 1, so 
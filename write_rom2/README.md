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
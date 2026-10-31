# top = main

from spade import SpadeExt
import cocotb
from cocotb.triggers import Timer

def read_reg(datapath, i):
    packed = int(datapath.regfile_0.x.value)
    return (packed >> (16 * i)) & 0xFFFF

@cocotb.test()
async def test_fetch(dut):
    clk = dut.clk_i
    datapath = dut.datapath_0
    ram = datapath.ram_0.ram

    clk.value = 0

    # Initial contents of the simulated RAM.
    ram[0].value = 0x51
    ram[1].value = 0x23

    # One reset edge, then four fetch edges.
    for _ in range(5):
        await Timer(5, "ns")
        clk.value = 1
        await Timer(5, "ns")
        clk.value = 0

    assert int(datapath.instr.value) == 0x5123
    assert int(datapath.pc.value) == 2

@cocotb.test()
async def test_lda(dut):
    s = SpadeExt(dut)
    print(s)
    clk = dut.clk_i
    datapath = dut.datapath_0
    ram = datapath.ram_0.ram

    clk.value = 0

    # Initial contents of the simulated RAM.
    ram[0].value = 0x10 # 0 
    ram[1].value = 0x02 # 1
    ram[2].value = 0xAB # 2
    ram[3].value = 0xCD # 3

    # One reset edge, then four fetch edges.
    for _ in range(5):
        await Timer(5, "ns")
        clk.value = 1
        await Timer(5, "ns")
        clk.value = 0


    # One reset edge, then four fetch edges.
    for _ in range(5):
        await Timer(5, "ns")
        clk.value = 1
        await Timer(5, "ns")
        clk.value = 0
    # import code
    # code.interact(local=locals())
    s.o.x.assert_eq([0xABCD, 0, 0,0])


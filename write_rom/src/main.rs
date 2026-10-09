// positions of control flags are measured from the lsb, so a_we is position 15 (the shift amount)
const A_WE: u16 = 15;
const B_WE: u16 = 14;
const C_WE: u16 = 13;
const MAR_WE: u16 = 12;
const RAM_WE: u16 = 11;
const PC_INC: u16 = 10;
const PC_JMP: u16 = 9;
const INST_WE: u16 = 8;
const BUS_SOURCE: u16 = 5; // shift amount 
const INST_STEP_RST: u16 = 4;
const HLT: u16 = 3;

enum Op {
    Nop = 0,
    Lda = 1,
    Add = 2,
    Sta = 4,
    Ldi = 5,
    Jmp = 6,
    Hlt = 15,
}
enum BusSource {
    A = 0,
    B = 1,
    C = 2,
    Alu = 3,
    Ram = 4,
    Pc = 5,
    Inst = 6,
}

/// A Control struct represents one microinstruction, that is, the collection of flags for a single step within an instruction like Lda
struct Control {
    a_we: bool,
    b_we: bool,
    c_we: bool,
    mar_we: bool,
    ram_we: bool,
    pc_inc: bool,
    pc_jmp: bool,
    inst_we: bool,
    bus_source: BusSource,
    inst_step_rst: bool,
    hlt: bool,
}
impl Default for Control {
    fn default() -> Self {
        Self {
            a_we: false,
            b_we: false,
            c_we: false,
            mar_we: false,
            ram_we: false,
            pc_inc: false,
            pc_jmp: false,
            inst_we: false,
            bus_source: BusSource::A,
            inst_step_rst: false,
            hlt: false,
        }
    }
}

// the execution steps omit the fetch instructions as they are common to all ops
fn execution_steps(o: Op) -> Vec<Control> {
    match o {
        Op::Nop => vec![Control::default()],
        Op::Lda => {
            // LDA 15, so inst_reg=0001 1111 and we
            vec![
                Control {
                    bus_source: BusSource::Inst,
                    mar_we: true,
                    ..Control::default()
                },
                Control {
                    bus_source: BusSource::Ram,
                    a_we: true,
                    ..Default::default()
                },
                Control {
                    inst_step_rst: true,
                    ..Default::default()
                },
            ]
        }
        Op::Add => {
            vec![
                Control {
                    bus_source: BusSource::Inst,
                    mar_we: true,
                    ..Control::default()
                },
                Control {
                    bus_source: BusSource::Ram,
                    b_we: true,
                    ..Default::default()
                },
                Control {
                    bus_source: BusSource::Alu,
                    a_we: true,
                    ..Default::default()
                },
                Control {
                    inst_step_rst: true,
                    ..Default::default()
                },
            ]
        }
        Op::Sta => {
            vec![
                Control {
                    bus_source: BusSource::Inst,
                    mar_we: true,
                    ..Control::default()
                },
                Control {
                    bus_source: BusSource::A,
                    ram_we: true,
                    ..Default::default()
                },
                Control {
                    inst_step_rst: true,
                    ..Default::default()
                },
            ]
        }
        Op::Ldi => {
            vec![
                Control {
                    bus_source: BusSource::Inst,
                    a_we: true,
                    ..Control::default()
                },
                Control {
                    inst_step_rst: true,
                    ..Default::default()
                },
            ]
        }
        Op::Jmp => {
            vec![
                Control {
                    bus_source: BusSource::Inst,
                    pc_jmp: true,
                    ..Control::default()
                },
                Control {
                    inst_step_rst: true,
                    ..Default::default()
                },
            ]
        }
        Op::Hlt => {
            vec![Control {
                hlt: true,
                ..Control::default()
            }]
        }
    }
}

fn encode(c: Control) -> u16 {
    (u16::from(c.a_we) << A_WE)
        | (u16::from(c.b_we) << B_WE)
        | (u16::from(c.c_we) << C_WE)
        | (u16::from(c.mar_we) << MAR_WE)
        | (u16::from(c.ram_we) << RAM_WE)
        | (u16::from(c.pc_inc) << PC_INC)
        | (u16::from(c.pc_jmp) << PC_JMP)
        | (u16::from(c.inst_we) << INST_WE)
        | ((c.bus_source as u16) << BUS_SOURCE)
        | (u16::from(c.inst_step_rst) << INST_STEP_RST)
        | (u16::from(c.hlt) << HLT)
}

fn main() {
    let mut mem: [u16; 256] = [0; 256];
    
}

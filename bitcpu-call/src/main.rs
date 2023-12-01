use std::fs::File;
use std::io::{self, Read};

enum CpuType {
    Nand,
    Nor,
    Xor,
    Xnor,
}

// 8 bit dstsrc, 8 bit src
//    or 16 bit jmpaddr
type Instr = u16;

fn parser(value: &str) -> u16 {
    if value.starts_with("0x") {
        u16::from_str_radix(value.strip_prefix("0x").unwrap(), 16).unwrap()
    } else {
        value.parse().unwrap()
    }
}

fn compiler(src: &str) -> (CpuType, Vec<Instr>) {
    let mut cputype = CpuType::Nand;
    let mut prog = vec![];
    for (i, line) in src.lines().enumerate() {
        let rowstart = line.split('#').next().unwrap().trim();
        if rowstart.is_empty() {
            continue;
        }
        if i == 0 {
            cputype = match rowstart {
                "NAND_CPU" => CpuType::Nand,
                "NOR_CPU" => CpuType::Nor,
                "XOR_CPU" => CpuType::Xor,
                "XNOR_CPU" => CpuType::Xnor,
                _ => {
                    eprintln!("First line: ADDLEQ or SUBLEQ");
                    std::process::exit(-1);
                }
            };
            continue;
        }
        let inst = parser(rowstart);
        prog.push(inst);
    }
    (cputype, prog)
}

// -- VCPU Runner --
struct Vcpu {
    outct: u8, // for formatted print!()
    cputype: CpuType,
    data: [bool; 256],
}

impl Vcpu {
    pub fn new(cputype: CpuType) -> Self {
        let data = [false; 256];
        Vcpu {
            outct: 0,
            cputype,
            data,
        }
    }

    fn getbit(&self) -> bool {
        loop {
            let mut inp: [u8; 1] = [0; 1];
            io::stdin().read_exact(&mut inp).expect("failed to read");
            if inp[0] == b'0' || inp[0] == b'1' {
                return (inp[0] - b'0') != 0;
            }
        }
    }

    fn putbit(&mut self, value: bool) {
        print!("{}", (0x30 + value as u8) as char);
        self.outct += 1;
        match self.outct {
            4 => print!(" "),
            8 => {
                print!("  ");
                self.outct = 0;
            }
            _ => (),
        }
    }

    // Memory & memory mapped functions
    fn mem_rd(&self, addr: u8) -> bool {
        let tf = match self.cputype {
            CpuType::Nand => true, // true & true --> false
            CpuType::Nor => false, // false | false --> true
            CpuType::Xor => false,
            CpuType::Xnor => false,
        };
        match addr {
            0x00..=0xfc => self.data[addr as usize], // generic RAM
            0xfd => self.getbit(),                   // stdin  - Read stdin,
            0xfe => tf,                              // stdout - Read const
            0xff => !tf,                             // read const
        }
    }

    // Memory & memory mapped functions
    fn mem_wr(&mut self, addr: u8, value: bool) {
        match addr {
            0x00..=0xfd => self.data[addr as usize] = value, // RAM, 0xfd write: JMP indicator
            0xfe => self.putbit(value),                      // stdout
            0xff => (),
        }
    }

    pub fn runner(&mut self, prog: &[Instr]) {
        let jmpflag_default = match self.cputype {
            CpuType::Nand => true, // true & true --> false
            CpuType::Nor => false, // false | false --> true
            CpuType::Xor => false,
            CpuType::Xnor => false,
        };
        self.mem_wr(0xfb, jmpflag_default); // no ret
        self.mem_wr(0xfc, jmpflag_default); // no jmp
        let mut pc_save = vec![];

        let mut pc = 0;
        // CPU run
        while pc < prog.len() {
            if self.mem_rd(0xfc) ^ jmpflag_default {
                pc = prog[pc] as usize; // jmp
                self.mem_wr(0xfc, jmpflag_default); // jmp flag reset
            } else {
                let rega = (prog[pc] >> 8) as u8;
                let regb = prog[pc] as u8;
                match self.cputype {
                    CpuType::Nand => self.mem_wr(rega, !(self.mem_rd(rega) & self.mem_rd(regb))),
                    CpuType::Nor => self.mem_wr(rega, !(self.mem_rd(rega) | self.mem_rd(regb))),
                    CpuType::Xor => self.mem_wr(rega, self.mem_rd(rega) ^ self.mem_rd(regb)),
                    CpuType::Xnor => self.mem_wr(rega, !(self.mem_rd(rega) ^ self.mem_rd(regb))),
                }
                if regb == 0xfc {
                    pc_save.push(pc);
                    self.mem_wr(0xfc, !jmpflag_default); // next: jmp
                }
                if rega == 0xfb && self.mem_rd(rega) ^ jmpflag_default {
                    pc = pc_save.pop().unwrap(); // return
                    self.mem_wr(rega, jmpflag_default); // ret flag reset
                }
            }
            pc += 1;
        }
    }
}

fn main() {
    if let Some(fname) = std::env::args().nth(1) {
        let mut file = File::open(fname).expect("program file not found");
        let mut src = String::new();
        file.read_to_string(&mut src).expect("failed to read");
        let (cputype, prog) = compiler(&src);
        let mut vcpu = Vcpu::new(cputype);
        vcpu.runner(&prog);
    } else {
        eprintln!("usage: nandcpu <file.bcpu>");
    }
}

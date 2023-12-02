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
type Instr = (u8, u8, u8);

fn parser(value: &str) -> u32 {
    if value.starts_with("0x") {
        u32::from_str_radix(value.strip_prefix("0x").unwrap(), 16).unwrap()
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
        prog.push(((inst >> 16) as u8, (inst >> 8) as u8, inst as u8));
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
        print!("{}", (0x30 + value as u8) as char); // inverse
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
        match addr {
            0x00..=0xfc => self.data[addr as usize], // generic RAM
            0xfd => self.getbit(),                   // stdin  - Read stdin,
            0xfe => false,                           // const false
            0xff => true,                            // const true
        }
    }

    // Memory & memory mapped functions
    fn mem_wr(&mut self, addr: u8, value: bool) {
        match addr {
            0x00..=0xfc => self.data[addr as usize] = value, // RAM
            0xfd => self.putbit(value),                      // stdout
            0xfe | 0xff => self.data[addr as usize] = value, // RET, JMP
        }
    }

    pub fn runner(&mut self, prog: &[Instr], trace: bool) {
        let mut pc_save = vec![];

        let mut pc = 0;
        // CPU run
        while pc < prog.len() {
            let (dst, src1, src2) = prog[pc];
            // trace (debug)
            if trace {
                let mut tracemem = String::new();
                for (i, &dbool) in self.data[0..0x80].iter().enumerate() {
                    if i % 4 == 0 {
                        tracemem.push(' ');
                    }
                    if i % 8 == 0 {
                        tracemem.push(' ');
                    }
                    let d = 0x30 + dbool as u8;
                    tracemem.push(d as char);
                }
                eprintln!("{pc:04x}: {dst:02x}, {src1:02x}, {src2:02x} mem:{tracemem}");
                if dst == 0xfe {
                    eprintln!("--- ret ---");
                }
            }
            // end of trace (debug)

            match self.cputype {
                CpuType::Nand => self.mem_wr(dst, !(self.mem_rd(src1) & self.mem_rd(src2))),
                CpuType::Nor => self.mem_wr(dst, !(self.mem_rd(src1) | self.mem_rd(src2))),
                CpuType::Xor => self.mem_wr(dst, self.mem_rd(src1) ^ self.mem_rd(src2)),
                CpuType::Xnor => self.mem_wr(dst, !(self.mem_rd(src1) ^ self.mem_rd(src2))),
            }
            if dst == 0xfe {
                pc = pc_save.pop().unwrap(); // return, postinc PC
            }
            pc += 1; // inc PC
                     // jump if true OR call
            if self.data[0xff] || (dst == 0xff && src2 == 0xfe) {
                self.data[0xff] = false;
                if src2 == 0xfe {
                    pc_save.push(pc); // call
                }
                let (a1, a2, a3) = prog[pc];
                if trace {
                    eprintln!("{pc:04x}: {a1:02x}, {a2:02x}, {a3:02x}");
                    eprintln!("--- jmp/call ---");
                }
                pc = (a1 as usize) << 16 | (a2 as usize) << 8 | a3 as usize;
            }
        }
    }
}

fn main() {
    if let Some(fname) = std::env::args().nth(1) {
        let mut file = File::open(fname).expect("program file not found");
        let mut src = String::new();
        file.read_to_string(&mut src).expect("failed to read");
        let mut trace = false;
        if let Some(param) = std::env::args().nth(2) {
            if param == "trace" {
                trace = true
            }
        }
        let (cputype, prog) = compiler(&src);
        let mut vcpu = Vcpu::new(cputype);
        vcpu.runner(&prog, trace);
    } else {
        eprintln!("usage: nandcpu <file.bcpu>");
    }
}

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
    cputype: CpuType,
    data: [bool; 256],
}

impl Vcpu {
    fn getbit(&self) -> bool {
        loop {
            let mut inp: [u8; 1] = [0; 1];
            io::stdin().read_exact(&mut inp).expect("failed to read");
            if inp[0] == b'0' || inp[0] == b'1' {
                return (inp[0] - b'0') != 0;
            }
        }
    }

    fn putbit(&self, value: bool) {
        print!("{}", (0x30 + value as u8) as char);
    }

    // Memory & memory mapped functions
    fn mem_rd(&self, addr: u8) -> bool {
        assert!(addr <= 0x0f);
        match addr {
            0x00..=0xfc => self.data[addr as usize], // generic RAM
            0xfd => self.getbit(),                   // stdin  - Read stdin,
            0xfe => true,                            // stdout - Read TRUE
            0xff => false,                           // ROM    - Read FALSE, Write jmp indicator
        }
    }

    // Memory & memory mapped functions
    fn mem_wr(&mut self, addr: u8, value: bool) {
        assert!(addr <= 0x0f);
        match addr {
            0xfe => self.putbit(value),            // stdout
            _ => self.data[addr as usize] = value, // RAM, 0x0f: JMP indicator
        }
    }

    pub fn new(cputype: CpuType) -> Self {
        let data = [false; 256];
        Vcpu { cputype, data }
    }

    pub fn runner_nandcpu(&mut self, prog: &[Instr]) {
        let mut pc = 0;
        // CPU run
        while pc < prog.len() {
            // Cond JMP, 0x0f reg & clr
            if self.mem_rd(0x0f) {
                pc = prog[pc] as usize;
                self.mem_wr(0xff, false); // reset jmp
            } else {
                let rega = (prog[pc] >> 8) as u8;
                let regb = prog[pc] as u8;
                match self.cputype {
                    CpuType::Nand => self.mem_wr(rega, !(self.mem_rd(rega) & self.mem_rd(regb))),
                    CpuType::Nor => self.mem_wr(rega, !(self.mem_rd(rega) | self.mem_rd(regb))),
                    CpuType::Xor => self.mem_wr(rega, self.mem_rd(rega) ^ self.mem_rd(regb)),
                    CpuType::Xnor => self.mem_wr(rega, !(self.mem_rd(rega) ^ self.mem_rd(regb))),
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
        vcpu.runner_nandcpu(&prog);
    } else {
        eprintln!("usage: nandcpu <file.bcpu>");
    }
}

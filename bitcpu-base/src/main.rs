use std::fs::File;
use std::io::{self, Read};

enum CpuType {
    Nand,
    Nor,
    Xor,
    Xnor,
}

// 8 bit dst, 8 bit src1, 8 bit src2
// by jmp addr = src1<<8 + src2
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
    io_func_outct: u8, // for formatted print!()
    cputype: CpuType,
    data: [bool; 256],
}

impl Vcpu {
    pub fn new(cputype: CpuType) -> Self {
        let data = [false; 256];
        Vcpu {
            io_func_outct: 0,
            cputype,
            data,
        }
    }

    fn trace_print(&self, pc: usize, dst: u8, src1: u8, src2: u8, trace: bool) {
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
        }
    }

    fn trace_print_jmp(&self, trace: bool) {
        if trace {
            eprintln!("--- jmp ---");
        }
    }

    fn io_getbit(&self) -> bool {
        loop {
            let mut inp: [u8; 1] = [0; 1];
            io::stdin().read_exact(&mut inp).expect("failed to read");
            if inp[0] == b'0' || inp[0] == b'1' {
                return (inp[0] - b'0') != 0;
            }
        }
    }

    fn io_putbit(&mut self, value: bool) {
        print!("{}", (0x30 + value as u8) as char); // inverse
        self.io_func_outct += 1;
        match self.io_func_outct {
            4 => print!(" "),
            8 => {
                print!("  ");
                self.io_func_outct = 0;
            }
            _ => (),
        }
    }

    // Memory & memory mapped functions
    fn mem_rd(&self, addr: u8) -> bool {
        match addr {
            0x00..=0xfc => self.data[addr as usize], // generic RAM
            0xfd => self.io_getbit(),                // stdin  - Read stdin,
            0xfe => false,                           // const GND
            0xff => true,                            // const +3v3
        }
    }

    // Memory & memory mapped functions
    fn mem_wr(&mut self, addr: u8, value: bool) {
        match addr {
            0xfd => self.io_putbit(value),         // stdout
            _ => self.data[addr as usize] = value, // RAM
        }
    }

    pub fn runner(&mut self, prog: &[Instr], trace: bool) {
        let mut pc = 0;
        // CPU run
        while pc < prog.len() {
            let (dst, src1, src2) = prog[pc];
            self.trace_print(pc, dst, src1, src2, trace); // trace for debug

            // ALU func
            let result = match self.cputype {
                CpuType::Nand => !(self.mem_rd(src1) & self.mem_rd(src2)),
                CpuType::Nor => !(self.mem_rd(src1) | self.mem_rd(src2)),
                CpuType::Xor => self.mem_rd(src1) ^ self.mem_rd(src2),
                CpuType::Xnor => !(self.mem_rd(src1) ^ self.mem_rd(src2)),
            };
            self.mem_wr(dst, result);
            // SKIP next instruction
            if dst == 0xfe && result {
                pc += 1;
            }
            // normal increment PC
            pc += 1;
            // JMP function
            if dst == 0xff {
                self.trace_print_jmp(trace);
                pc = (src1 as usize) << 8 | src2 as usize;
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

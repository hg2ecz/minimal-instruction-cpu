use std::fs::File;
use std::io::{self, Read};

enum CpuType {
    Addleq,
    Subleq,
}
// addrA addrB jmpaddr
type Instr = (u8, u8, i16);

fn parser(value: &str) -> i16 {
    if value.starts_with("0x") {
        i16::from_str_radix(value.strip_prefix("0x").unwrap(), 16).unwrap()
    } else if value.starts_with("-0x") {
        -i16::from_str_radix(value.strip_prefix("-0x").unwrap(), 16).unwrap()
    } else {
        value.parse().unwrap()
    }
}

fn compiler(src: &str) -> (CpuType, Vec<Instr>, Vec<i16>) {
    let mut cputype = CpuType::Subleq;
    let mut prog = vec![];
    let mut rom = vec![];
    for (i, line) in src.lines().enumerate() {
        let rowstart = line.split('#').next().unwrap().trim();
        if rowstart.len() < 5 {
            continue;
        }
        if i == 0 {
            cputype = match rowstart {
                "ADDLEQ" => CpuType::Addleq,
                "SUBLEQ" => CpuType::Subleq,
                _ => {
                    eprintln!("First line: ADDLEQ or SUBLEQ");
                    std::process::exit(-1);
                }
            };
            continue;
        }
        let mut token = rowstart.split_whitespace();
        let first = token.next().unwrap();
        if first == "rom" {
            for x in token {
                rom.push(parser(x));
            }
        } else {
            let addr_a = parser(first) as u8;
            let addr_b = parser(token.next().unwrap()) as u8;
            let jmpaddr = parser(token.next().unwrap());
            prog.push((addr_a, addr_b, jmpaddr));
        }
    }
    (cputype, prog, rom)
}

// -- VCPU Runner --

fn getchar() -> i16 {
    let mut inp: [u8; 1] = [0; 1];
    io::stdin().read_exact(&mut inp).expect("failed to read");
    inp[0] as i16
}

fn putchar(value: i16) {
    print!("{}", char::from_u32(value as u32).unwrap());
}

struct Vcpu {
    cputype: CpuType,
    data: [i16; 256],
}

impl Vcpu {
    // Memory & memory mapped functions
    fn mem_rd(&self, addr: u8) -> i16 {
        match addr {
            0x00..=0xfd => self.data[addr as usize], // RAM, ROM
            0xfe => getchar(),                       // Stdin
            0xff => 0,                               // Stdout, read 0
        }
    }

    // Memory & memory mapped functions
    fn mem_wr(&mut self, addr: u8, value: i16) {
        match addr {
            0x00..=0x7f => self.data[addr as usize] = value, // RAM, last: readable stdout
            0x80..=0xfe => (),                               // ROM write not allowed
            0xff => putchar(value),                          // Stdout
        }
    }

    pub fn new(cputype: CpuType) -> Self {
        let data: [i16; 256] = [0; 256];
        Vcpu { cputype, data }
    }

    pub fn runner(&mut self, prog: &[Instr], rom: &[i16]) {
        self.data[0x80..0x80 + rom.len()].copy_from_slice(rom);
        let mut pc = 0;
        // CPU run
        while pc < prog.len() {
            let instr = prog[pc];
            let result = match self.cputype {
                CpuType::Addleq => self.mem_rd(instr.0) + self.mem_rd(instr.1),
                CpuType::Subleq => self.mem_rd(instr.0) - self.mem_rd(instr.1),
            };
            self.mem_wr(instr.0, result);
            if result <= 0 {
                pc += instr.2 as usize;
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
        let (cputype, prog, rom) = compiler(&src);
        let mut vcpu = Vcpu::new(cputype);
        vcpu.runner(&prog, &rom);
    } else {
        eprintln!("usage: subleq <file.subleq>");
    }
}

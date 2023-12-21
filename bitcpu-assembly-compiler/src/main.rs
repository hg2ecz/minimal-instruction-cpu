use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::BufWriter;
use std::io::Write;
use std::iter::FromIterator;
use std::path::Path;

const DEBUG: bool = true;

// comment: # and ;
// hexnum
// decnum
// a equ 0x03
// label:
// a = nand(a, b)
// jmp label
// jnz_nand(a, b) label

fn help() {
    println!("Valid instructions:");
    println!("   label:                 ; for address labels");
    println!("   dest equ 12            ; for datareg labels");
    println!("   a    equ 0x0c          ; for datareg labels");
    println!("   dest = nand(a, b)      ; nand with labels");
    println!("   0x0c = nand(0xff, 12)  ; nand with address");
    println!("   skip_nand(a, b)        ; skip next instruction");
    println!("   jmp addr, call addr, ret");
}

fn splitter(s_in: &str) -> Vec<String> {
    let mut words: Vec<String> = vec![];
    let mut wstart = false;
    let mut chars = vec![];
    for ch in s_in.to_lowercase().chars() {
        if [';', '#'].contains(&ch) {
            break;
        }
        if [' ', '\t', '(', ',', ')'].contains(&ch) {
            if wstart {
                words.push(String::from_iter(chars.clone()));
                chars.clear();
                wstart = false;
            }
        } else {
            wstart = true;
            chars.push(ch);
        }
    }
    if wstart {
        words.push(String::from_iter(chars.clone()));
    }
    words
}

fn parsenum(s: &str, linenum: usize) -> u32 {
    if s.starts_with("0x") {
        if let Ok(num) = u32::from_str_radix(s.strip_prefix("0x").unwrap(), 16) {
            num
        } else {
            eprintln!("Syntax error in line {} (parsehex)", linenum + 1);
            std::process::exit(1);
        }
    } else if let Ok(num) = s.parse::<u32>() {
        num
    } else {
        eprintln!("Syntax error in line {} (parsedec)", linenum + 1);
        std::process::exit(1);
    }
}

fn equ_get(equ_hmap: &HashMap<String, u32>, keyword: &str, linenum: usize) -> u32 {
    if let Some(d) = equ_hmap.get(keyword) {
        d & 0xff
    } else {
        parsenum(keyword, linenum) & 0xff
    }
}

// Preprocessing: macro and included file
fn preprocessor_include(
    assembly_code: &str,
    filename: &str,
    fnamelist: &mut Vec<String>,
) -> String {
    let parentdir = Path::new(filename)
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    let mut linearized = String::new();
    for s in assembly_code.lines() {
        let words = splitter(s);
        if !words.is_empty() && words[0] == "%include" {
            let fname = parentdir.clone() + "/" + &words[1].replace('"', "");
            if fnamelist.contains(&fname) {
                eprintln!("This file ({fname}) was before included! Exit.");
                std::process::exit(1);
            }
            fnamelist.push(fname.clone());
            let inner = preprocessor_include(
                &fs::read_to_string(&fname).expect("File not found."),
                &fname,
                fnamelist,
            );
            linearized.push_str(&inner);
        } else {
            linearized.push_str(s);
            linearized.push('\n');
        }
    }
    linearized
}

fn preprocessor_macro(assembly_code: &str) -> String {
    let mut macro_hash = HashMap::new();
    let mut linearized = String::new();
    struct MacroStruct {
        macro_codes: String,
        macro_argnum: u8,
        reference_num: u32,
    }
    let mut macro_mode = false;
    let mut macro_name = String::new();
    let mut macro_argnum = 0;
    let mut macro_codes = String::new();
    for s in assembly_code.lines() {
        let words = splitter(s);
        if !words.is_empty() {
            match words[0].as_str() {
                "%macro" => {
                    macro_name = words[1].clone();
                    macro_argnum = words[2].parse().unwrap();
                    macro_mode = true;
                }
                "%endmacro" => {
                    let mstr = MacroStruct {
                        macro_argnum,
                        macro_codes: macro_codes.clone(),
                        reference_num: 0,
                    };
                    macro_hash.insert(macro_name.clone(), mstr);
                    macro_codes.clear();
                    macro_mode = false;
                }
                _ => {
                    if let Some(macro_data) = macro_hash.get(&words[0]) {
                        // insert_macro
                        linearized.push_str(&("; macro ".to_owned() + &words[0] + "\n"));
                        linearized.push_str(&macro_data.macro_codes);
                        linearized.push_str(&("; endmacro".to_owned() + &words[0] + "\n"));
                    } else if macro_mode {
                        macro_codes.push_str(s);
                        macro_codes.push('\n');
                    } else {
                        linearized.push_str(s);
                        linearized.push('\n');
                    }
                }
            }
        }
    }
    linearized
}

// Ccompile "linearized" file (here is not include and macro)
fn assembler(assembly_code: &str) -> (String, Vec<u32>) {
    let mut cpu_type = String::new();
    let mut machine_code = vec![];
    let mut addr_labels = HashMap::new();
    let mut equ_labels = HashMap::new();
    let mut address = 0;

    // Stage-1: Process address labels (for forward jmp)
    for (linenum, line) in assembly_code.lines().enumerate() {
        if linenum == 0 {
            let cpu_types = ["nand_cpu"];
            let words: Vec<_> = splitter(line);
            if cpu_types.iter().any(|e| words[0].contains(e)) {
                cpu_type = words[0].to_uppercase();
            } else {
                eprintln!("First line must be one of these: {:?}", cpu_types);
                help();
                std::process::exit(1);
            }
        } else if !line.is_empty() {
            if line.ends_with(':') {
                let label = line.trim_end_matches(':').to_string();
                addr_labels.insert(label.to_lowercase(), address);
            } else {
                let words: Vec<_> = splitter(line);
                if (!words.is_empty()
                    && ["skip_nand", "jmp", "call", "ret"]
                        .iter()
                        .any(|e| words[0].contains(e)))
                    || (words.len() > 1 && (words[1] == "=" && words[2] == "nand"))
                {
                    address += 1;
                }
            }
        }
    }

    if DEBUG {
        println!("Debug addr_labels: {addr_labels:?}");
    }

    // Stage-2: Generate machine code
    for (linenum, line) in assembly_code.lines().enumerate() {
        if linenum > 0 && !line.is_empty() && !line.ends_with(':') {
            let words = splitter(line);
            if words.is_empty() {
                continue;
            }
            if (["skip_nand", "jmp", "call", "ret"]
                .iter()
                .any(|e| words[0].contains(e)))
                || (words.len() >= 2
                    && ((words[1] == "=" && words[2] == "nand") || words[1] == "equ"))
            {
                if DEBUG {
                    println!("Debug: {:?} --> {:?}", line, words);
                }
                if words.len() == 3 && words[1] == "equ" {
                    equ_labels.insert(words[0].clone(), parsenum(&words[2], linenum));
                } else if words[0] == "skip_nand" {
                    let a = equ_get(&equ_labels, &words[1], linenum);
                    let b = equ_get(&equ_labels, &words[2], linenum);
                    machine_code.push(0xfe << 16 | a << 8 | b);
                } else if words[0] == "jmp" {
                    let address = if let Some(&addr) = addr_labels.get(&words[1]) {
                        addr
                    } else {
                        parsenum(&words[1], linenum)
                    };
                    machine_code.push(0xff0000 | address);
                } else if words[0] == "call" {
                    let address = if let Some(&addr) = addr_labels.get(&words[1]) {
                        addr
                    } else {
                        parsenum(&words[1], linenum)
                    };
                    machine_code.push(0xfc0000 | address);
                } else if words[0] == "ret" {
                    machine_code.push(0xfc0000); // address 0x0000 start, not callable
                } else if words[1] == "=" && words[2] == "nand" {
                    let d = equ_get(&equ_labels, &words[0], linenum);
                    let a = equ_get(&equ_labels, &words[3], linenum);
                    let b = equ_get(&equ_labels, &words[4], linenum);
                    machine_code.push(d << 16 | a << 8 | b);
                } else {
                    eprintln!("Syntax error in line {linenum} (not a valid token syntax)");
                    std::process::exit(1);
                }
            } else {
                eprintln!("Syntax error in line {linenum} (unknown token)");
                help();
                std::process::exit(1);
            }
        }
    }
    (cpu_type, machine_code)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let basename = Path::new(filename)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    let assembly_code = fs::read_to_string(filename).expect("File not found.");
    let mut filenamevec = vec![];
    let assembly_code = preprocessor_include(&assembly_code, filename, &mut filenamevec);
    let assembly_code = preprocessor_macro(&assembly_code);
    if DEBUG {
        for s in assembly_code.lines() {
            println!("{s}");
        }
    }
    let (cpu_type, machine_code) = assembler(&assembly_code);

    if DEBUG {
        for (i, code) in machine_code.iter().enumerate() {
            println!("Debug code({i:4}): {:06x}", code);
        }
    }

    let file = fs::File::create(basename + ".lst").unwrap();
    let mut writer = BufWriter::new(file);
    writeln!(&mut writer, "{cpu_type}").unwrap();
    for code in machine_code {
        writeln!(&mut writer, "0x{code:06x}").unwrap();
    }
    writer.flush().unwrap();
}

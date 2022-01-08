mod cpu;
mod memory;

use std::env;
use std::fs::File;
use std::io::{self, Read};

use crate::cpu::*;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("Usage: mips-emu filename>");
    }
    let mut file = File::open(&args[1])?;
    let mut binary = Vec::new();
    file.read_to_end(&mut binary)?;
    
    let mut cpu = Processor::new();

    cpu.write_to_bfm(&binary);

    loop {
        let inst = match cpu.get_inst() {
            Ok(inst) => inst,
            Err(_) => break,
        };

        let branch = cpu.handle_instruction(inst);

        if !branch {
            cpu.pc = cpu.pc + 4;
        }

        if cpu.pc == 0 {
            break;
        }
    }

    Ok(())
}

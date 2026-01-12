#![allow(warnings)]
mod hardware;
mod utils;

use std::sync::{Arc, Mutex, mpsc};

use hardware::{
    architecture::Palabra, disk::Disk, interrupts::Interrups, ram::Ram, registers::Registros,
};

use crate::{
    hardware::{
        cpu::Cpu,
        dma::{Dma, Dma_Config},
        instructions::{self, Instruction},
        interrupts::{External_interrupt, handle_interrupt},
    },
    utils::convert_to_string_format_pal,
};

fn main() {
    let mut ram = Arc::new(Mutex::new(Ram::new()));
    let mut external_interrupts = Arc::new(Mutex::new(External_interrupt::new()));
    let (tx, rx) = mpsc::channel::<Dma_Config>();
    {
        let mut men = ram.lock().unwrap();
        for i in (0..9) {
            let code_interrupt = Palabra::new(&format!("9{i}000000").to_string()).unwrap();
            men.writeMemory(i, code_interrupt);
        }

        men.writeMemory(300, Palabra::new(&"04100001".to_string()).unwrap());
        men.writeMemory(301, Palabra::new(&"25100010".to_string()).unwrap());
        men.writeMemory(302, Palabra::new(&"13000000".to_string()).unwrap());

        // men.writeMemory(300, Palabra::new(&"05200002".to_string()).unwrap());
    }

    let mut cpu = Cpu::new(Arc::clone(&ram), Arc::clone(&external_interrupts), tx);

    cpu.registers
        .set_rb(Palabra::new(&"00000300".to_string()).unwrap());
    cpu.registers
        .set_rl(Palabra::new(&"00000306".to_string()).unwrap());
    cpu.registers.psw.set_pc(300);
    cpu.registers.rx = Palabra::new(&"00000304".to_string()).unwrap();

    cpu.registers.sp = Palabra::new(&"00000306".to_string()).unwrap();

    cpu.run();

    // cpu(Arc::clone(&ram), Arc::clone(&external_interrupts), tx);
}

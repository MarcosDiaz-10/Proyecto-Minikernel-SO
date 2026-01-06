mod hardware;
mod utils;

use std::sync::{Arc, Mutex};

use hardware::{
    architecture::Palabra, cpu::Registros, disk::Disk, interrupts::Interrups, ram::Ram,
};

use crate::hardware::{cpu::ExternalInterrup, interrupts};

fn main() {
    let mut registers = Registros::new();
    let mut memRar = Arc::new(Mutex::new(Ram::new()));

    // registers.psw.set_codition(0);

    // {
    //     let mut stateMem = memRar.lock().unwrap();

    //     stateMem
    //         .writeMemory(0, Palabra::new(&String::from("10010000")).unwrap())
    //         .unwrap();
    //     let pal = stateMem.readMemory(0).unwrap();
    //     println!("{}", pal.convert());
    // }
    // let arr = [1, 2, 3];

    // let mut disk = Disk::new();

    // let res = disk.read(0, 0, 0).unwrap();
    // let pal = Palabra::new(&String::from("10010000")).unwrap();
    // disk.write(pal.convert_to_string_disk(), 0, 0, 0).unwrap();
    // println!("{}", res);
    // {
    //     let stateMem = memRar.lock().unwrap();
    //     let pal = stateMem.readMemory(0).unwrap();

    //     println!("{}", pal.convert());
    // }
}

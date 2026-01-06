#![allow(warnings)]
mod hardware;
mod utils;

use std::sync::{Arc, Mutex};

use hardware::{
    architecture::Palabra, disk::Disk, interrupts::Interrups, ram::Ram, registers::Registros,
};

use crate::hardware::{
    dma::Dma,
    interrupts::{ExternalInterrup, handle_interrupt},
};

fn main() {
    let mut registers = Registros::new();
    let mut memRar = Arc::new(Mutex::new(Ram::new()));

    let mut external_interrupsts = Arc::new(Mutex::new(ExternalInterrup::new()));

    let mut disk = Disk::new();
    let pal = Palabra::new(&String::from("10050000")).unwrap();

    {
        let mut m = memRar.lock().unwrap();

        m.writeMemory(301, pal);
    }

    let mut dma = Dma::new();

    dma.cil_acceder = 2;
    dma.pista_acceder = 10;
    dma.sector_acceder = 100;
    dma.pos_men = 301;

    let result_write = dma.write_disk(
        &mut disk,
        Arc::clone(&memRar),
        Arc::clone(&external_interrupsts),
    );

    match result_write {
        Ok(_) => (),
        Err(err) => println!("{:?}", err),
    };

    {
        let m = external_interrupsts.lock().unwrap();

        println!("{:?}", m);
    }

    handle_interrupt(
        &mut registers,
        Interrups::EndIO,
        Arc::clone(&memRar),
        Arc::clone(&external_interrupsts),
    );

    {
        let m = external_interrupsts.lock().unwrap();

        println!("{:?}", m);
    }

    let result = dma.read_disk(
        &disk,
        Arc::clone(&memRar),
        Arc::clone(&external_interrupsts),
    );

    match result {
        Ok(_) => (),
        Err(err) => println!("{:?}", err),
    };

    {
        let m = memRar.lock().unwrap();

        println!("Lectura memoria: {:?}", m.readMemory(301).unwrap());
    }

    {
        let m = external_interrupsts.lock().unwrap();

        println!("{:?}", m);
    }

    handle_interrupt(
        &mut registers,
        Interrups::EndIO,
        Arc::clone(&memRar),
        Arc::clone(&external_interrupsts),
    );

    {
        let m = external_interrupsts.lock().unwrap();

        println!("{:?}", m);
    }

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

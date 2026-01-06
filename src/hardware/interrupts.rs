use std::sync::{Arc, Mutex};

use crate::{
    hardware::{
        architecture::Palabra,
        cpu::{ExternalInterrup, Registros},
        ram::Ram,
    },
    utils::ContinueOrBreak,
};

#[derive(Debug)]
pub enum Interrups {
    Overflow = 8,
    Underflow = 7,
    DirInv = 6,
    InstInv = 5,
    EndIO = 4,
    Clock = 3,
    CallSys = 2,
    CodIntInv = 1,
    CodCallSysInv = 0,
}

pub fn overflow(regs: &mut Registros) -> ContinueOrBreak {
    regs.psw.set_codition(3).unwrap();
    ContinueOrBreak::Break
}

pub fn underflow(regs: &mut Registros) -> ContinueOrBreak {
    regs.psw.set_codition(3).unwrap();
    ContinueOrBreak::Break
}

pub fn dir_inv() -> ContinueOrBreak {
    println!("Direccionamiento Invalido");
    ContinueOrBreak::Break
}

pub fn inst_inv() -> ContinueOrBreak {
    println!("Instrucción Invalido");
    ContinueOrBreak::Break
}

pub fn end_io(externalInterrup: &mut ExternalInterrup) -> ContinueOrBreak {
    externalInterrup.int_io = true;

    ContinueOrBreak::Continue
}

pub fn clock(externalInterrup: &mut ExternalInterrup) -> ContinueOrBreak {
    externalInterrup.int_reloj = true;
    ContinueOrBreak::Continue
}

pub fn call_sys(regs: &Registros, ram: Arc<Mutex<Ram>>) -> ContinueOrBreak {
    let codCall = regs.ac.convert();

    let parametro: Palabra;
    {
        let state_ram = ram.lock().unwrap();

        parametro = state_ram.readMemory(regs.sp.convert()).unwrap();
    }

    match codCall {
        _ => println!("Llamada al sistema {codCall}, parametro{:?}", parametro),
    }
    ContinueOrBreak::Continue
}

pub fn cod_int_inv() -> ContinueOrBreak {
    ContinueOrBreak::Break
}

pub fn cod_call_sys_inv() -> ContinueOrBreak {
    ContinueOrBreak::Break
}

pub fn handle_interrupt(
    regs: &mut Registros,
    cod_int: Interrups,
    ram: Arc<Mutex<Ram>>,
    external_int: &mut ExternalInterrup,
) -> ContinueOrBreak {
    match cod_int {
        Interrups::Overflow => overflow(regs),
        Interrups::Underflow => underflow(regs),
        Interrups::DirInv => dir_inv(),
        Interrups::InstInv => inst_inv(),
        Interrups::EndIO => end_io(external_int),
        Interrups::Clock => clock(external_int),
        Interrups::CallSys => call_sys(regs, ram),
        Interrups::CodIntInv => cod_int_inv(),
        Interrups::CodCallSysInv => cod_call_sys_inv(),
    }
}

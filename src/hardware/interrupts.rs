use std::{
    marker,
    sync::{Arc, Mutex},
};

use crate::{
    hardware::{architecture::Palabra, ram::Ram, registers::Registros},
    utils::ContinueOrBreak,
};
#[derive(Debug)]
pub struct External_interrupt {
    pub int_overflow: bool,
    pub int_underflow: bool,
    pub int_dir_inv: bool,
    pub int_inst_inv: bool,
    pub int_io: bool,
    pub int_clock: bool,
    pub int_call_sys: bool,
    pub int_cod_inte_inv: bool,
    pub int_cod_callsys_inv: bool,
}

impl External_interrupt {
    pub fn new() -> Self {
        External_interrupt {
            int_overflow: false,
            int_underflow: false,
            int_dir_inv: false,
            int_inst_inv: false,
            int_io: false,
            int_clock: false,
            int_call_sys: false,
            int_cod_inte_inv: false,
            int_cod_callsys_inv: false,
        }
    }
}

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
    ContinueOrBreak::Break
}

pub fn underflow(regs: &mut Registros) -> ContinueOrBreak {
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

pub fn end_io() -> ContinueOrBreak {
    println!("Termino I/O");
    ContinueOrBreak::Continue
}

pub fn clock() -> ContinueOrBreak {
    println!("Clock");
    ContinueOrBreak::Continue
}

pub fn call_sys(
    regs: &Registros,
    ram: Arc<Mutex<Ram>>,
    external_int: Arc<Mutex<External_interrupt>>,
) -> ContinueOrBreak {
    let codCall = regs.ac.convert();

    let parametro;
    {
        let state_ram = ram.lock().unwrap();

        parametro = state_ram.readMemory(regs.sp.convert());
    }

    let parametro_pal = match parametro {
        Ok(pal) => pal,
        Err(_) => {
            {
                external_int.lock().unwrap().int_dir_inv = true;
            }
            return ContinueOrBreak::Continue;
        }
    };

    match codCall {
        1 => {
            println!(
                "Llamada al sistema {} , parametro: {}",
                codCall,
                parametro_pal.convert()
            );

            return ContinueOrBreak::Break;
        }
        _ => {
            println!(
                "Llamada al sistema no encontrada {codCall}, parametro{:?}",
                parametro
            );

            {
                external_int.lock().unwrap().int_cod_callsys_inv = true;
            }
        }
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
    external_int: Arc<Mutex<External_interrupt>>,
) -> ContinueOrBreak {
    match cod_int {
        Interrups::Overflow => {
            {
                let mut lock_int = external_int.lock().unwrap();
                lock_int.int_overflow = false;
            }

            overflow(regs)
        }
        Interrups::Underflow => {
            {
                let mut lock_int = external_int.lock().unwrap();
                lock_int.int_underflow = false;
            }

            underflow(regs)
        }
        Interrups::DirInv => {
            {
                let mut lock_int = external_int.lock().unwrap();
                lock_int.int_dir_inv = false;
            }

            dir_inv()
        }
        Interrups::InstInv => {
            {
                let mut lock_int = external_int.lock().unwrap();
                lock_int.int_inst_inv = false;
            }

            inst_inv()
        }
        Interrups::EndIO => {
            {
                let mut lock_int = external_int.lock().unwrap();
                lock_int.int_io = false;
            }

            end_io()
        }
        Interrups::Clock => {
            {
                let mut lock_int = external_int.lock().unwrap();
                lock_int.int_clock = false;
            }

            clock()
        }
        Interrups::CallSys => {
            {
                let mut lock_int = external_int.lock().unwrap();
                lock_int.int_call_sys = false;
            }

            call_sys(regs, ram, external_int)
        }
        Interrups::CodIntInv => {
            {
                let mut lock_int = external_int.lock().unwrap();
                lock_int.int_cod_inte_inv = false;
            }

            cod_int_inv()
        }
        Interrups::CodCallSysInv => {
            {
                let mut lock_int = external_int.lock().unwrap();
                lock_int.int_cod_callsys_inv = false;
            }

            cod_call_sys_inv()
        }
    }
}

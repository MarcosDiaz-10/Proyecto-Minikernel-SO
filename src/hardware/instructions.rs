use std::{
    mem,
    sync::{Arc, Mutex, mpsc::Sender},
};

use crate::{
    hardware::{
        architecture::Palabra, dma::Dma, interrupts::Interrups, ram::Ram, registers::Registros,
    },
    utils::{
        Errors, Result_op, convert_option_result, convert_result, convert_to_string_format_pal,
    },
};
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Instruction {
    pub opcode: u8,
    pub dir: u8,
    pub value: u32,
}

impl Instruction {
    pub fn new(pal: Palabra) -> Self {
        Instruction {
            opcode: (pal.palabra / 1000000) as u8,
            dir: ((pal.palabra / 100000) % 10) as u8,
            value: (pal.palabra % 100000) as u32,
        }
    }

    pub fn conver_to_palabra(self) -> Palabra {
        Palabra::new(&format!("{:02}{:01}{:05}", self.opcode, self.dir, self.value).to_string())
            .unwrap()
    }
    // pub fn sum(&self, regs: &mut Registros) -> Result_op {
    //     let pal = (regs.ac + regs.mdr);

    //     match pal {
    //         Ok(palabra) => Ok(regs.ac = palabra),
    //         Err(err) => {
    //             regs.psw.set_codition(3)?;
    //             return Err(err);
    //         }
    //     }
    // }

    // pub fn rest(&self, regs: &mut Registros) -> Result_op {
    //     let pal = (regs.ac - regs.mdr);

    //     match pal {
    //         Ok(palabra) => Ok(regs.ac = palabra),
    //         Err(err) => {
    //             regs.psw.set_codition(3)?;
    //             return Err(err);
    //         }
    //     }
    // }
    // pub fn mult(&self, regs: &mut Registros) -> Result_op {
    //     let pal = (regs.ac * regs.mdr);

    //     match pal {
    //         Ok(palabra) => Ok(regs.ac = palabra),
    //         Err(err) => {
    //             regs.psw.set_codition(3)?;
    //             return Err(err);
    //         }
    //     }
    // }
    // pub fn divi(&self, regs: &mut Registros) -> Result_op {
    //     let pal = (regs.ac / regs.mdr);

    //     match pal {
    //         Ok(palabra) => Ok(regs.ac = palabra),
    //         Err(err) => {
    //             regs.psw.set_codition(3)?;
    //             return Err(err);
    //         }
    //     }
    // }

    // pub fn load(self, regs: &mut Registros) -> Result_op {
    //     regs.ac = regs.mdr;
    //     Ok(())
    // }

    // pub fn store(self, regs: &Registros, mem: Arc<Mutex<Ram>>) -> Result_op {
    //     let state_mem = mem.lock();
    //     let mut state_mem = convert_result(
    //         state_mem,
    //         "Error con la sincronización del bus".to_string(),
    //         Interrups::DirInv,
    //     )?;

    //     state_mem.writeMemory(regs.mar.convert(), regs.mdr)?;

    //     Ok(())
    // }

    // pub fn load_rx(self, regs: &mut Registros) -> Result_op {
    //     regs.ac = regs.rx;
    //     Ok(())
    // }
    // pub fn store_rx(self, regs: &mut Registros) -> Result_op {
    //     regs.rx = regs.ac;
    //     Ok(())
    // }

    // pub fn comp(self, regs: &mut Registros) -> Result_op {
    //     let comp = (regs.ac - regs.mdr)?.convert();

    //     if comp == 0 {
    //         regs.psw.set_codition(0)?;
    //     } else if comp < 0 {
    //         regs.psw.set_codition(1)?;
    //     } else if comp > 0 {
    //         regs.psw.set_codition(2)?;
    //     }
    //     Ok(())
    // }

    // pub fn jmpe(self, regs: &mut Registros, mem: Arc<Mutex<Ram>>) -> Result_op {
    //     let state_mem = mem.lock();
    //     let mut state_mem = convert_result(
    //         state_mem,
    //         "Error con la sincronización del bus".to_string(),
    //         Interrups::DirInv,
    //     )?;

    //     let memory_readed = state_mem.readMemory(regs.sp.convert())?;

    //     if regs.ac == memory_readed {
    //         regs.psw.set_pc(regs.mdr.convert())?;
    //     }

    //     Ok(())
    // }
    // pub fn jmpne(self, regs: &mut Registros, mem: Arc<Mutex<Ram>>) -> Result_op {
    //     let state_mem = mem.lock();
    //     let mut state_mem = convert_result(
    //         state_mem,
    //         "Error con la sincronización del bus".to_string(),
    //         Interrups::DirInv,
    //     )?;

    //     let memory_readed = state_mem.readMemory(regs.sp.convert())?;

    //     if regs.ac != memory_readed {
    //         regs.psw.set_pc(regs.mdr.convert())?;
    //     }

    //     Ok(())
    // }
    // pub fn jmplt(self, regs: &mut Registros, mem: Arc<Mutex<Ram>>) -> Result_op {
    //     let state_mem = mem.lock();
    //     let mut state_mem = convert_result(
    //         state_mem,
    //         "Error con la sincronización del bus".to_string(),
    //         Interrups::DirInv,
    //     )?;

    //     let memory_readed = state_mem.readMemory(regs.sp.convert())?;

    //     if regs.ac < memory_readed {
    //         regs.psw.set_pc(regs.mdr.convert())?;
    //     }

    //     Ok(())
    // }
    // pub fn jmplgt(self, regs: &mut Registros, mem: Arc<Mutex<Ram>>) -> Result_op {
    //     let state_mem = mem.lock();
    //     let mut state_mem = convert_result(
    //         state_mem,
    //         "Error con la sincronización del bus".to_string(),
    //         Interrups::DirInv,
    //     )?;

    //     let memory_readed = state_mem.readMemory(regs.sp.convert())?;

    //     if regs.ac > memory_readed {
    //         regs.psw.set_pc(regs.mdr.convert())?;
    //     }

    //     Ok(())
    // }

    // pub fn svc(self, regs: &Registros) -> Errors {
    //     Errors {
    //         msg: "Llamada al sistema".to_string(),
    //         cod: Interrups::CallSys,
    //     }
    // }

    // pub fn psh(self, regs: &mut Registros, mem: Arc<Mutex<Ram>>) -> Result_op {
    //     let new_sp = (regs.sp - Palabra::new("00000001").unwrap())?;
    //     if (new_sp < regs.rx) {
    //         return Err(Errors {
    //             msg: "Stack Overflow".to_string(),
    //             cod: Interrups::InstInv,
    //         });
    //     }

    //     let state_mem = mem.lock();
    //     let mut state_mem = convert_result(
    //         state_mem,
    //         "Error con la sincronización del bus".to_string(),
    //         Interrups::DirInv,
    //     )?;

    //     state_mem.writeMemory(regs.sp.convert(), regs.ac)?;

    //     regs.sp = new_sp;
    //     Ok(())
    // }

    // pub fn pop(self, regs: &mut Registros, mem: Arc<Mutex<Ram>>) -> Result_op {
    //     let new_sp = (regs.sp + Palabra::new("00000001").unwrap())?;

    //     let state_mem = mem.lock();
    //     let mut state_mem = convert_result(
    //         state_mem,
    //         "Error con la sincronización del bus".to_string(),
    //         Interrups::DirInv,
    //     )?;

    //     let value_stack = state_mem.readMemory(regs.sp.convert())?;
    //     regs.ac = value_stack;
    //     regs.sp = new_sp;
    //     Ok(())
    // }

    // pub fn retrn(self, regs: &mut Registros, mem: Arc<Mutex<Ram>>) -> Result_op {
    //     self.pop(regs, mem)?;

    //     regs.psw.set_pc(regs.ac.convert())?;

    //     Ok(())
    // }

    // pub fn hab(self, regs: &mut Registros) -> Result_op {
    //     if regs.psw.modo_op == 0 {
    //         return Err(Errors {
    //             msg: "Falta de privilegios, se necesita modo kernel".to_string(),
    //             cod: Interrups::InstInv,
    //         });
    //     }

    //     regs.psw.set_inte(1);

    //     Ok(())
    // }
    // pub fn dhab(self, regs: &mut Registros) -> Result_op {
    //     if regs.psw.modo_op == 0 {
    //         return Err(Errors {
    //             msg: "Falta de privilegios, se necesita modo kernel".to_string(),
    //             cod: Interrups::InstInv,
    //         });
    //     }

    //     regs.psw.set_inte(0);

    //     Ok(())
    // }

    // pub fn tti(self, regs: &mut Registros, clock: &mut u32) -> Result_op {
    //     if regs.psw.modo_op == 0 {
    //         return Err(Errors {
    //             msg: "Falta de privilegios, se necesita modo kernel".to_string(),
    //             cod: Interrups::InstInv,
    //         });
    //     }
    //     let data = regs.mdr.convert();
    //     if data < 0 {
    //         return Err(Errors {
    //             msg: "El numero de reloj no puede ser negativo".to_string(),
    //             cod: Interrups::InstInv,
    //         });
    //     }
    //     *clock = data as u32;
    //     Ok(())
    // }

    // pub fn chmod(self, regs: &mut Registros) -> Result_op {
    //     if regs.psw.modo_op == 0 {
    //         return Err(Errors {
    //             msg: "Falta de privilegios, se necesita modo kernel".to_string(),
    //             cod: Interrups::InstInv,
    //         });
    //     }

    //     regs.psw.set_mode(1);

    //     Ok(())
    // }

    // pub fn load_rb(self, regs: &mut Registros) -> Result_op {
    //     regs.ac = regs.rb;
    //     Ok(())
    // }

    // pub fn store_rb(self, regs: &mut Registros) -> Result_op {
    //     regs.set_rb(regs.ac)?;
    //     Ok(())
    // }
    // pub fn load_rl(self, regs: &mut Registros) -> Result_op {
    //     regs.ac = regs.rl;
    //     Ok(())
    // }
    // pub fn store_rl(self, regs: &mut Registros) -> Result_op {
    //     regs.set_rl(regs.ac)?;
    //     Ok(())
    // }
    // pub fn load_sp(self, regs: &mut Registros) -> Result_op {
    //     regs.ac = regs.sp;
    //     Ok(())
    // }

    // pub fn store_sp(self, regs: &mut Registros) -> Result_op {
    //     regs.set_sp(regs.ac)?;
    //     Ok(())
    // }

    // //Esto permite que se hagan saltos indirecto, es decir, cuando el modo de direccionamiento sea distinto a inmediato. Lo que va a suceder es que la dirección se comporta como un puntero
    // pub fn j(self, regs: &mut Registros) -> Result_op {
    //     regs.psw.set_pc(regs.mdr.convert())?;
    //     Ok(())
    // }

    // pub fn sdmap(self, regs: &Registros, dma: &mut Dma) -> Result_op {
    //     dma.pista_acceder = regs.mdr.convert() as i8;
    //     Ok(())
    // }
    // pub fn sdmac(self, regs: &Registros, dma: &mut Dma) -> Result_op {
    //     dma.cil_acceder = regs.mdr.convert() as i8;
    //     Ok(())
    // }
    // pub fn sdmas(self, regs: &Registros, dma: &mut Dma) -> Result_op {
    //     dma.sector_acceder = regs.mdr.convert() as i8;
    //     Ok(())
    // }
    // pub fn sdmaio(self, regs: &Registros, dma: &mut Dma) -> Result_op {
    //     dma.modo = regs.mdr.convert() as i8;
    //     Ok(())
    // }
    // pub fn sdmam(self, regs: &Registros, dma: &mut Dma) -> Result_op {
    //     dma.pos_men = regs.mdr.convert();
    //     Ok(())
    // }
    // pub fn sdmaon(self, regs: &Registros, dma: Dma, sender: Sender<Dma>) -> Result_op {
    //     convert_result(
    //         sender.send(dma),
    //         "Error al enviar orden dma".to_string(),
    //         Interrups::InstInv,
    //     )?;
    //     Ok(())
    // }
}

use std::{
    sync::{Arc, Mutex, MutexGuard},
    thread,
    time::Duration,
};

use crate::{
    hardware::{
        architecture::Palabra,
        disk::Disk,
        interrupts::{External_interrupt, Interrups},
        ram::Ram,
    },
    utils::{Errors, Result_op, convert_option_result, convert_result},
};
#[derive(Debug, Clone, Copy, PartialEq)]

pub enum State_Dma {
    Succes,
    Error,
    Off,
}

#[derive(Debug, Clone, Copy)]
pub struct Dma_Config {
    pub pista_acceder: i8,
    pub sector_acceder: i8,
    pub cil_acceder: i8,
    pub pos_men: i32,
    pub state: State_Dma,
    pub modo: i8,
}

impl Dma_Config {
    pub fn new() -> Self {
        Dma_Config {
            pista_acceder: 0,
            sector_acceder: 0,
            cil_acceder: 0,
            pos_men: 0,
            state: State_Dma::Succes,
            modo: 0,
        }
    }
}
pub struct Dma {
    pub pista_acceder: i8,
    pub sector_acceder: i8,
    pub cil_acceder: i8,
    pub pos_men: i32,
    pub estado: State_Dma,
    pub modo: i8,
}

impl Dma {
    pub fn new() -> Self {
        Dma {
            pista_acceder: 0,
            sector_acceder: 0,
            cil_acceder: 0,
            pos_men: 0,
            estado: State_Dma::Succes,
            modo: 0,
        }
    }

    pub fn execute(
        &mut self,
        disk: &mut Disk,
        mem: &Arc<Mutex<Ram>>,
        external_interrup: &Arc<Mutex<External_interrupt>>,
    ) -> Result_op {
        let modo = self.modo;
        if modo == 0 {
            self.read_disk(disk, mem, external_interrup)?;
        } else if modo == 1 {
            self.write_disk(disk, mem, external_interrup)?;
        }

        Ok(())
    }

    pub fn read_disk(
        &mut self,
        disk: &Disk,
        mem: &Arc<Mutex<Ram>>,
        external_interrup: &Arc<Mutex<External_interrupt>>,
    ) -> Result_op {
        thread::sleep(Duration::from_secs(1));
        let result = disk.read(self.cil_acceder, self.pista_acceder, self.sector_acceder);

        let result = match result {
            Err(err) => {
                self.estado = State_Dma::Error;
                return Err(err);
            }
            Ok(r) => r,
        };

        {
            let state_mem_result = mem.lock();
            let mut state_mem = match state_mem_result {
                Ok(val) => val,
                Err(_) => {
                    self.estado = State_Dma::Error;

                    return Err(Errors {
                        msg: "Error con la sincronización del bus".to_string(),
                        cod: Interrups::EndIO,
                    });
                }
            };

            let new_pal = convert_option_result(
                Palabra::new(&result),
                "Error al transformar la palabra del disco".to_string(),
                Interrups::InstInv,
            );

            let new_pal = match new_pal {
                Ok(p) => p,
                Err(err) => {
                    self.estado = State_Dma::Error;
                    return Err(err);
                }
            };

            let result_write_mem = state_mem.writeMemory(self.pos_men, new_pal);
            match result_write_mem {
                Err(err) => {
                    self.estado = State_Dma::Error;
                    return Err(err);
                }
                _ => (),
            }
        }

        {
            let state_external_interrup_result = external_interrup.lock();
            let mut state_external_interrup = match state_external_interrup_result {
                Ok(i) => i,
                Err(_) => {
                    self.estado = State_Dma::Error;
                    return Err(Errors {
                        msg: "Error lanzando la interrupción".to_string(),
                        cod: Interrups::InstInv,
                    });
                }
            };
            state_external_interrup.int_io = true;
        }

        Ok(())
    }

    pub fn write_disk(
        &mut self,
        disk: &mut Disk,
        mem: &Arc<Mutex<Ram>>,
        external_interrup: &Arc<Mutex<External_interrupt>>,
    ) -> Result_op {
        thread::sleep(Duration::from_secs(1));
        let pal_disk: String;

        {
            let state_mem_result = mem.lock();
            let state_mem = match state_mem_result {
                Ok(val) => val,
                Err(_) => {
                    self.estado = State_Dma::Error;

                    return Err(Errors {
                        msg: "Error con la sincronización del bus".to_string(),
                        cod: Interrups::EndIO,
                    });
                }
            };

            let pal_read_mem = state_mem.readMemory(self.pos_men);
            let pal_read_mem = match pal_read_mem {
                Ok(p) => p,
                Err(err) => {
                    self.estado = State_Dma::Error;

                    return Err(err);
                }
            };

            pal_disk = pal_read_mem.convert_to_string_disk();
        }

        let result_write = disk.write(
            pal_disk,
            self.cil_acceder,
            self.pista_acceder,
            self.sector_acceder,
        );

        match result_write {
            Err(err) => {
                self.estado = State_Dma::Error;

                return Err(err);
            }
            _ => (),
        }

        {
            let state_external_interrup_result = external_interrup.lock();
            let mut state_external_interrup = match state_external_interrup_result {
                Ok(i) => i,
                Err(_) => {
                    self.estado = State_Dma::Error;
                    return Err(Errors {
                        msg: "Error lanzando la interrupción".to_string(),
                        cod: Interrups::InstInv,
                    });
                }
            };
            state_external_interrup.int_io = true;
        }

        Ok(())
    }
}

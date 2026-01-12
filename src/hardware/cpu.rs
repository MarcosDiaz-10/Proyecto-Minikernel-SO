use std::sync::{Arc, Mutex, mpsc::Sender};

use crate::{
    hardware::{
        architecture::Palabra,
        dma::{Dma, Dma_Config},
        instructions::Instruction,
        interrupts::{External_interrupt, Interrups, handle_interrupt},
        ram::Ram,
        registers::{self, Registros},
    },
    utils::{
        ContinueOrBreak, Errors, Result_op, convert_option_result, convert_result,
        convert_to_string_format_pal,
    },
};

#[derive(Debug)]
pub struct Cpu {
    pub registers: Registros,
    ram: Arc<Mutex<Ram>>,
    pub external_interrupt: Arc<Mutex<External_interrupt>>,
    sender_dma: Sender<Dma_Config>,
    pub clock_interrupt: u32,
    pub dma_temp: Dma_Config,
    pub have_user_program: bool,
}

impl Cpu {
    pub fn new(
        ram: Arc<Mutex<Ram>>,
        external_interrupt: Arc<Mutex<External_interrupt>>,
        sender_dma: Sender<Dma_Config>,
    ) -> Self {
        Cpu {
            registers: Registros::new(),
            clock_interrupt: 3,
            dma_temp: Dma_Config::new(),
            have_user_program: false,
            ram,
            external_interrupt,
            sender_dma,
        }
    }
    pub fn run(&mut self) {
        self.have_user_program = true;
        while self.have_user_program {
            self.step();
            println!("Registers: {:#?}", self.registers)
        }
    }

    pub fn step(&mut self) {
        match self.fetch_decode_execute() {
            Ok(()) => (),
            Err(E) => match E.cod {
                Interrups::Overflow => self.external_interrupt.lock().unwrap().int_overflow = true,
                Interrups::Underflow => {
                    self.external_interrupt.lock().unwrap().int_underflow = true
                }
                Interrups::DirInv => self.external_interrupt.lock().unwrap().int_dir_inv = true,
                Interrups::InstInv => self.external_interrupt.lock().unwrap().int_inst_inv = true,
                Interrups::CallSys => self.external_interrupt.lock().unwrap().int_call_sys = true,
                Interrups::Clock => self.external_interrupt.lock().unwrap().int_clock = true,
                Interrups::CodCallSysInv => {
                    self.external_interrupt.lock().unwrap().int_cod_callsys_inv = true
                }
                Interrups::CodIntInv => {
                    self.external_interrupt.lock().unwrap().int_cod_inte_inv = true
                }
                Interrups::EndIO => self.external_interrupt.lock().unwrap().int_io = true,
            },
        }
        self.vector_interrupt();
    }

    fn vector_interrupt(&mut self) -> Result_op {
        let ext = self.external_interrupt.lock().unwrap();

        //Falta salvaguarda de estado para algunas instrucciones en el cambio de contexto

        if ext.int_overflow {
            self.registers.psw.set_mode(1)?;
            self.registers.psw.pc = 8;
            return Ok(());
        }

        if ext.int_underflow {
            self.registers.psw.set_mode(1)?;

            self.registers.psw.pc = 7;
            return Ok(());
        }
        if ext.int_dir_inv {
            self.registers.psw.set_mode(1)?;

            self.registers.psw.pc = 6;
            return Ok(());
        }
        if ext.int_inst_inv {
            self.registers.psw.set_mode(1)?;

            self.registers.psw.pc = 5;
            return Ok(());
        }
        if ext.int_io {
            self.registers.psw.set_mode(1)?;

            self.registers.psw.pc = 4;
            return Ok(());
        }
        if ext.int_clock {
            self.registers.psw.set_mode(1)?;

            self.registers.psw.pc = 3;
            return Ok(());
        }
        if ext.int_call_sys {
            self.registers.psw.set_mode(1)?;

            self.registers.psw.pc = 2;
            return Ok(());
        }
        if ext.int_cod_inte_inv {
            self.registers.psw.set_mode(1)?;
            self.registers.psw.pc = 1;
            return Ok(());
        }
        if ext.int_cod_callsys_inv {
            self.registers.psw.set_mode(1)?;
            self.registers.psw.pc = 0;
            return Ok(());
        }

        Ok(())
    }

    fn fetch_decode_execute(&mut self) -> Result_op {
        //Fetch
        self.fetch()?;
        //Decode
        self.decode()?;

        //Execute
        self.execute()?;
        Ok(())
    }

    fn fetch(&mut self) -> Result_op {
        let state_mem_result = self.ram.lock();
        let mut state_mem = match state_mem_result {
            Ok(val) => val,
            Err(_) => {
                {
                    let mut ex = self.external_interrupt.lock().unwrap();
                    ex.int_inst_inv = true;
                }

                return Err(Errors {
                    msg: "Error con la sincronización del bus".to_string(),
                    cod: Interrups::InstInv,
                });
            }
        };

        let pos_mem_palabra = convert_option_result(
            Palabra::new(&convert_to_string_format_pal(self.registers.psw.pc)),
            "Error al transformar palabra pc".to_string(),
            Interrups::InstInv,
        )?;

        if self.registers.psw.modo_op != 1 {
            if pos_mem_palabra < self.registers.rb || pos_mem_palabra > self.registers.rl {
                return Err(Errors {
                    msg: "Fuera de los límites de memoria".to_string(),
                    cod: Interrups::DirInv,
                });
            }
        }

        self.registers.set_mar(pos_mem_palabra)?;
        self.registers.mdr = state_mem.readMemory(self.registers.psw.pc)?;
        self.registers.ir = Instruction::new(self.registers.mdr);
        self.registers.psw.pc += 1;
        Ok(())
    }

    fn decode(&mut self) -> Result_op {
        if self.registers.ir.opcode != 5 {
            match self.registers.ir.dir {
                0 => self.dir_direct()?,
                1 => self.dir_inmediate()?,
                2 => self.dir_indexed()?,
                _ => {
                    return Err(Errors {
                        msg: "Codigo de direccionamiento inválido".to_string(),
                        cod: Interrups::DirInv,
                    });
                }
            }
        } else {
            match self.registers.ir.dir {
                1 => self.dir_direct_store()?,
                2 => self.dir_indexed_store()?,
                _ => {
                    return Err(Errors {
                        msg: "Codigo de direccionamiento inválido store".to_string(),
                        cod: Interrups::DirInv,
                    });
                }
            }
        }
        Ok(())
    }

    fn execute(&mut self) -> Result_op {
        match self.registers.ir.opcode {
            0 => self.sum()?,
            1 => self.rest()?,
            2 => self.mult()?,
            3 => self.divi()?,
            4 => self.load()?,
            5 => self.store()?,
            6 => self.load_rx()?,
            7 => self.store_rx()?,
            8 => self.comp()?,
            9 => self.jmpe()?,
            10 => self.jmpne()?,
            11 => self.jmplt()?,
            12 => self.jmplgt()?,
            13 => self.svc()?,
            14 => self.retrn()?,
            15 => self.hab()?,
            16 => self.dhab()?,
            17 => self.tti()?,
            18 => self.chmod()?,
            19 => self.load_rb()?,
            20 => self.store_rb()?,
            21 => self.load_rl()?,
            22 => self.store_rl()?,
            23 => self.load_sp()?,
            24 => self.store_sp()?,
            25 => self.psh()?,
            26 => self.pop()?,
            27 => self.j()?,
            28 => self.sdmap()?,
            29 => self.sdmac()?,
            30 => self.sdmas()?,
            31 => self.sdmaio()?,
            32 => self.sdmam()?,
            33 => self.sdmaon()?,
            90 => {
                let response_handle = handle_interrupt(
                    &mut self.registers,
                    Interrups::CodCallSysInv,
                    Arc::clone(&self.ram),
                    Arc::clone(&self.external_interrupt),
                );

                self.chmod();
                match response_handle {
                    ContinueOrBreak::Break => self.have_user_program = false,
                    ContinueOrBreak::Continue => (),
                }
            }
            91 => {
                let response_handle = handle_interrupt(
                    &mut self.registers,
                    Interrups::CodIntInv,
                    Arc::clone(&self.ram),
                    Arc::clone(&self.external_interrupt),
                );
                self.chmod();

                match response_handle {
                    ContinueOrBreak::Break => self.have_user_program = false,
                    ContinueOrBreak::Continue => (),
                }
            }
            92 => {
                let response_handle = handle_interrupt(
                    &mut self.registers,
                    Interrups::CallSys,
                    Arc::clone(&self.ram),
                    Arc::clone(&self.external_interrupt),
                );
                self.chmod();
                match response_handle {
                    ContinueOrBreak::Break => self.have_user_program = false,
                    ContinueOrBreak::Continue => (),
                }
            }
            93 => {
                let response_handle = handle_interrupt(
                    &mut self.registers,
                    Interrups::Clock,
                    Arc::clone(&self.ram),
                    Arc::clone(&self.external_interrupt),
                );
                self.chmod();

                match response_handle {
                    ContinueOrBreak::Break => self.have_user_program = false,
                    ContinueOrBreak::Continue => (),
                }
            }
            94 => {
                let response_handle = handle_interrupt(
                    &mut self.registers,
                    Interrups::EndIO,
                    Arc::clone(&self.ram),
                    Arc::clone(&self.external_interrupt),
                );
                self.chmod();

                match response_handle {
                    ContinueOrBreak::Break => self.have_user_program = false,
                    ContinueOrBreak::Continue => (),
                }
            }
            95 => {
                let response_handle = handle_interrupt(
                    &mut self.registers,
                    Interrups::InstInv,
                    Arc::clone(&self.ram),
                    Arc::clone(&self.external_interrupt),
                );
                self.chmod();

                match response_handle {
                    ContinueOrBreak::Break => self.have_user_program = false,
                    ContinueOrBreak::Continue => (),
                }
            }
            96 => {
                let response_handle = handle_interrupt(
                    &mut self.registers,
                    Interrups::DirInv,
                    Arc::clone(&self.ram),
                    Arc::clone(&self.external_interrupt),
                );
                self.chmod();

                match response_handle {
                    ContinueOrBreak::Break => self.have_user_program = false,
                    ContinueOrBreak::Continue => (),
                }
            }
            97 => {
                let response_handle = handle_interrupt(
                    &mut self.registers,
                    Interrups::Underflow,
                    Arc::clone(&self.ram),
                    Arc::clone(&self.external_interrupt),
                );
                self.chmod();

                match response_handle {
                    ContinueOrBreak::Break => self.have_user_program = false,
                    ContinueOrBreak::Continue => (),
                }
            }
            98 => {
                let response_handle = handle_interrupt(
                    &mut self.registers,
                    Interrups::Overflow,
                    Arc::clone(&self.ram),
                    Arc::clone(&self.external_interrupt),
                );
                self.chmod();

                match response_handle {
                    ContinueOrBreak::Break => self.have_user_program = false,
                    ContinueOrBreak::Continue => (),
                }
            }
            _ => {
                return Err(Errors {
                    msg: "Instrucción invalida execute".to_string(),
                    cod: Interrups::InstInv,
                });
            }
        }
        Ok(())
    }

    fn dir_direct(&mut self) -> Result_op {
        let dir_num: i32 = match self.registers.psw.modo_op {
            1 => self.registers.ir.value as i32,
            other => self.registers.rb.convert() + self.registers.ir.value as i32,
        };

        if self.registers.psw.modo_op == 1 {
            if dir_num > 2000 || dir_num < 0 {
                return Err(Errors {
                    msg: "Direccionamiento Invalido ".to_string(),
                    cod: Interrups::DirInv,
                });
            }
        } else {
            if dir_num > self.registers.rl.convert() {
                return Err(Errors {
                    msg: "Direccionamiento Invalido Overflow".to_string(),
                    cod: Interrups::DirInv,
                });
            }
        }

        let dir = convert_option_result(
            Palabra::new(&convert_to_string_format_pal(dir_num)),
            "Dir invalida".to_string(),
            Interrups::DirInv,
        )?;

        self.registers.set_mar(dir)?;
        {
            let state_mem_result = self.ram.lock();
            let mut state_mem = match state_mem_result {
                Ok(val) => val,
                Err(_) => {
                    {
                        let mut ex = self.external_interrupt.lock().unwrap();
                        ex.int_inst_inv = true;
                    }

                    return Err(Errors {
                        msg: "Error con la sincronización del bus".to_string(),
                        cod: Interrups::InstInv,
                    });
                }
            };

            let value = state_mem.readMemory(self.registers.mar.convert())?;
            self.registers.set_mdr(value);
        }
        Ok(())
    }
    fn dir_direct_store(&mut self) -> Result_op {
        let dir_num: i32 = match self.registers.psw.modo_op {
            1 => self.registers.ir.value as i32,
            other => self.registers.rb.convert() + self.registers.ir.value as i32,
        };

        if self.registers.psw.modo_op == 1 {
            if dir_num > 2000 || dir_num < 0 {
                return Err(Errors {
                    msg: "Direccionamiento Invalido ".to_string(),
                    cod: Interrups::DirInv,
                });
            }
        } else {
            if dir_num > self.registers.rl.convert() {
                return Err(Errors {
                    msg: "Direccionamiento Invalido Overflow".to_string(),
                    cod: Interrups::DirInv,
                });
            }
        }

        let dir = convert_option_result(
            Palabra::new(&convert_to_string_format_pal(dir_num)),
            "Dir invalida".to_string(),
            Interrups::DirInv,
        )?;

        self.registers.set_mar(dir)?;
        self.registers.set_mdr(self.registers.ac);
        Ok(())
    }
    fn dir_indexed_store(&mut self) -> Result_op {
        let index_dir = self.registers.ir.value as i32 + self.registers.ac.convert();
        let dir_num: i32 = match self.registers.psw.modo_op {
            1 => index_dir,
            other => self.registers.rb.convert() + index_dir,
        };

        if self.registers.psw.modo_op == 1 {
            if dir_num > 2000 || dir_num < 0 {
                return Err(Errors {
                    msg: "Direccionamiento Invalido ".to_string(),
                    cod: Interrups::DirInv,
                });
            }
        } else {
            if dir_num > self.registers.rl.convert() {
                return Err(Errors {
                    msg: "Direccionamiento Invalido Overflow".to_string(),
                    cod: Interrups::DirInv,
                });
            }
        }

        let dir = convert_option_result(
            Palabra::new(&convert_to_string_format_pal(dir_num)),
            "Dir invalida".to_string(),
            Interrups::DirInv,
        )?;

        self.registers.set_mar(dir)?;
        self.registers.set_mdr(self.registers.ac);

        Ok(())
    }
    fn dir_inmediate(&mut self) -> Result_op {
        let value = convert_option_result(
            Palabra::new(&convert_to_string_format_pal(
                self.registers.ir.value as i32,
            )),
            "Dir invalida".to_string(),
            Interrups::DirInv,
        )?;

        self.registers.set_mdr(value);

        Ok(())
    }

    fn dir_indexed(&mut self) -> Result_op {
        let index_dir = self.registers.ir.value as i32 + self.registers.ac.convert();
        let dir_num: i32 = match self.registers.psw.modo_op {
            1 => index_dir,
            other => self.registers.rb.convert() + index_dir,
        };

        if self.registers.psw.modo_op == 1 {
            if dir_num > 2000 || dir_num < 0 {
                return Err(Errors {
                    msg: "Direccionamiento Invalido ".to_string(),
                    cod: Interrups::DirInv,
                });
            }
        } else {
            if dir_num > self.registers.rl.convert() {
                return Err(Errors {
                    msg: "Direccionamiento Invalido Overflow".to_string(),
                    cod: Interrups::DirInv,
                });
            }
        }
        let dir = convert_option_result(
            Palabra::new(&convert_to_string_format_pal(dir_num)),
            "Dir invalida".to_string(),
            Interrups::DirInv,
        )?;

        self.registers.set_mar(dir)?;
        {
            let state_mem_result = self.ram.lock();
            let mut state_mem = match state_mem_result {
                Ok(val) => val,
                Err(_) => {
                    {
                        let mut ex = self.external_interrupt.lock().unwrap();
                        ex.int_inst_inv = true;
                    }

                    return Err(Errors {
                        msg: "Error con la sincronización del bus".to_string(),
                        cod: Interrups::InstInv,
                    });
                }
            };

            let value = state_mem.readMemory(self.registers.mar.convert())?;
            self.registers.set_mdr(value);
        }
        Ok(())
    }

    pub fn sum(&mut self) -> Result_op {
        let pal = (self.registers.ac + self.registers.mdr);

        match pal {
            Ok(palabra) => Ok(self.registers.ac = palabra),
            Err(err) => {
                self.registers.psw.set_codition(3)?;
                return Err(err);
            }
        }
    }

    pub fn rest(&mut self) -> Result_op {
        let pal = (self.registers.ac - self.registers.mdr);

        match pal {
            Ok(palabra) => Ok(self.registers.ac = palabra),
            Err(err) => {
                self.registers.psw.set_codition(3)?;
                return Err(err);
            }
        }
    }
    pub fn mult(&mut self) -> Result_op {
        let pal = (self.registers.ac * self.registers.mdr);

        match pal {
            Ok(palabra) => Ok(self.registers.ac = palabra),
            Err(err) => {
                self.registers.psw.set_codition(3)?;
                return Err(err);
            }
        }
    }
    pub fn divi(&mut self) -> Result_op {
        let pal = (self.registers.ac / self.registers.mdr);

        match pal {
            Ok(palabra) => Ok(self.registers.ac = palabra),
            Err(err) => {
                self.registers.psw.set_codition(3)?;
                return Err(err);
            }
        }
    }

    pub fn load(&mut self) -> Result_op {
        self.registers.ac = self.registers.mdr;
        Ok(())
    }

    pub fn store(&mut self) -> Result_op {
        let state_mem = self.ram.lock();
        let mut state_mem = convert_result(
            state_mem,
            "Error con la sincronización del bus".to_string(),
            Interrups::DirInv,
        )?;

        state_mem.writeMemory(self.registers.mar.convert(), self.registers.mdr)?;

        Ok(())
    }

    pub fn load_rx(&mut self) -> Result_op {
        self.registers.ac = self.registers.rx;
        Ok(())
    }
    pub fn store_rx(&mut self) -> Result_op {
        self.registers.rx = self.registers.ac;
        Ok(())
    }

    pub fn comp(&mut self) -> Result_op {
        let comp = (self.registers.ac - self.registers.mdr)?.convert();

        if comp == 0 {
            self.registers.psw.set_codition(0)?;
        } else if comp < 0 {
            self.registers.psw.set_codition(1)?;
        } else if comp > 0 {
            self.registers.psw.set_codition(2)?;
        }
        Ok(())
    }

    pub fn jmpe(&mut self) -> Result_op {
        let state_mem = self.ram.lock();
        let mut state_mem = convert_result(
            state_mem,
            "Error con la sincronización del bus".to_string(),
            Interrups::DirInv,
        )?;

        let memory_readed = state_mem.readMemory(self.registers.sp.convert())?;

        if self.registers.ac == memory_readed {
            self.registers.psw.set_pc(self.registers.mdr.convert())?;
        }

        Ok(())
    }
    pub fn jmpne(&mut self) -> Result_op {
        let state_mem = self.ram.lock();
        let mut state_mem = convert_result(
            state_mem,
            "Error con la sincronización del bus".to_string(),
            Interrups::DirInv,
        )?;

        let memory_readed = state_mem.readMemory(self.registers.sp.convert())?;

        if self.registers.ac != memory_readed {
            self.registers.psw.set_pc(self.registers.mdr.convert())?;
        }

        Ok(())
    }
    pub fn jmplt(&mut self) -> Result_op {
        let state_mem = self.ram.lock();
        let mut state_mem = convert_result(
            state_mem,
            "Error con la sincronización del bus".to_string(),
            Interrups::DirInv,
        )?;

        let memory_readed = state_mem.readMemory(self.registers.sp.convert())?;

        if self.registers.ac < memory_readed {
            self.registers.psw.set_pc(self.registers.mdr.convert())?;
        }

        Ok(())
    }
    pub fn jmplgt(&mut self) -> Result_op {
        let state_mem = self.ram.lock();
        let mut state_mem = convert_result(
            state_mem,
            "Error con la sincronización del bus".to_string(),
            Interrups::DirInv,
        )?;

        let memory_readed = state_mem.readMemory(self.registers.sp.convert())?;

        if self.registers.ac > memory_readed {
            self.registers.psw.set_pc(self.registers.mdr.convert())?;
        }

        Ok(())
    }

    pub fn svc(&self) -> Result_op {
        Err(Errors {
            msg: "Llamada al sistema".to_string(),
            cod: Interrups::CallSys,
        })
    }

    pub fn psh(&mut self) -> Result_op {
        let new_sp = (self.registers.sp - Palabra::new("00000001").unwrap())?;
        if (new_sp < self.registers.rx) {
            return Err(Errors {
                msg: "Stack Overflow".to_string(),
                cod: Interrups::InstInv,
            });
        }

        let state_mem = self.ram.lock();
        let mut state_mem = convert_result(
            state_mem,
            "Error con la sincronización del bus".to_string(),
            Interrups::DirInv,
        )?;

        state_mem.writeMemory(new_sp.convert(), self.registers.ac)?;

        self.registers.sp = new_sp;
        Ok(())
    }

    pub fn pop(&mut self) -> Result_op {
        let new_sp = (self.registers.sp + Palabra::new("00000001").unwrap())?;

        let state_mem = self.ram.lock();
        let mut state_mem = convert_result(
            state_mem,
            "Error con la sincronización del bus".to_string(),
            Interrups::DirInv,
        )?;

        let value_stack = state_mem.readMemory(self.registers.sp.convert())?;
        self.registers.ac = value_stack;
        self.registers.sp = new_sp;
        Ok(())
    }

    pub fn retrn(&mut self) -> Result_op {
        self.pop()?;

        self.registers.psw.set_pc(self.registers.ac.convert())?;

        Ok(())
    }

    pub fn hab(&mut self) -> Result_op {
        if self.registers.psw.modo_op == 0 {
            return Err(Errors {
                msg: "Falta de privilegios, se necesita modo kernel".to_string(),
                cod: Interrups::InstInv,
            });
        }

        self.registers.psw.set_inte(1);

        Ok(())
    }
    pub fn dhab(&mut self) -> Result_op {
        if self.registers.psw.modo_op == 0 {
            return Err(Errors {
                msg: "Falta de privilegios, se necesita modo kernel".to_string(),
                cod: Interrups::InstInv,
            });
        }

        self.registers.psw.set_inte(0);

        Ok(())
    }

    pub fn tti(&mut self) -> Result_op {
        if self.registers.psw.modo_op == 0 {
            return Err(Errors {
                msg: "Falta de privilegios, se necesita modo kernel".to_string(),
                cod: Interrups::InstInv,
            });
        }
        let data = self.registers.mdr.convert();
        if data < 0 {
            return Err(Errors {
                msg: "El numero de reloj no puede ser negativo".to_string(),
                cod: Interrups::InstInv,
            });
        }
        self.clock_interrupt = data as u32;
        Ok(())
    }

    pub fn chmod(&mut self) -> Result_op {
        if self.registers.psw.modo_op == 0 {
            return Err(Errors {
                msg: "Falta de privilegios, se necesita modo kernel".to_string(),
                cod: Interrups::InstInv,
            });
        }

        self.registers.psw.set_mode(0);

        Ok(())
    }

    pub fn load_rb(&mut self) -> Result_op {
        self.registers.ac = self.registers.rb;
        Ok(())
    }

    pub fn store_rb(&mut self) -> Result_op {
        self.registers.set_rb(self.registers.ac)?;
        Ok(())
    }
    pub fn load_rl(&mut self) -> Result_op {
        self.registers.ac = self.registers.rl;
        Ok(())
    }
    pub fn store_rl(&mut self) -> Result_op {
        self.registers.set_rl(self.registers.ac)?;
        Ok(())
    }
    pub fn load_sp(&mut self) -> Result_op {
        self.registers.ac = self.registers.sp;
        Ok(())
    }

    pub fn store_sp(&mut self) -> Result_op {
        self.registers.set_sp(self.registers.ac)?;
        Ok(())
    }

    //Esto permite que se hagan saltos indirecto, es decir, cuando el modo de direccionamiento sea distinto a inmediato. Lo que va a suceder es que la dirección se comporta como un puntero
    pub fn j(&mut self) -> Result_op {
        self.registers.psw.set_pc(self.registers.mdr.convert())?;
        Ok(())
    }

    pub fn sdmap(&mut self) -> Result_op {
        self.dma_temp.pista_acceder = self.registers.mdr.convert() as i8;
        Ok(())
    }
    pub fn sdmac(&mut self) -> Result_op {
        self.dma_temp.cil_acceder = self.registers.mdr.convert() as i8;
        Ok(())
    }
    pub fn sdmas(&mut self) -> Result_op {
        self.dma_temp.sector_acceder = self.registers.mdr.convert() as i8;
        Ok(())
    }
    pub fn sdmaio(&mut self) -> Result_op {
        self.dma_temp.modo = self.registers.mdr.convert() as i8;
        Ok(())
    }
    pub fn sdmam(&mut self) -> Result_op {
        self.dma_temp.pos_men = self.registers.mdr.convert();
        Ok(())
    }
    pub fn sdmaon(&mut self) -> Result_op {
        convert_result(
            self.sender_dma.send(self.dma_temp),
            "Error al enviar orden dma".to_string(),
            Interrups::InstInv,
        )?;
        Ok(())
    }
}

use std::{
    sync::{Arc, Mutex, mpsc::Sender},
    thread::Thread,
    thread::sleep,
    time::Duration,
};

use crate::{
    Mode_Execute,
    hardware::{
        architecture::Palabra,
        dma::{Dma, Dma_Config},
        instructions::Instruction,
        interrupts::{External_interrupt, Interrups, handle_interrupt},
        ram::Ram,
        registers::{self, Pws, Registros},
    },
    utils::{
        ContinueOrBreak, Errors, Result_op, convert_option_result, convert_result,
        convert_to_string_format_pal,
    },
};
#[derive(Debug, Clone, Copy)]
pub enum Result_Execute_program {
    Succes,
    Error,
}
#[derive(Debug, Clone)]

pub enum Result_Instruction {
    Palabra(Palabra),
    String(String),
}
#[derive(Debug, Clone)]

pub struct Result_Execute {
    pub result_program: Result_Execute_program,
    pub dir_inst: i32,
    pub instruction: Palabra,
    pub result_instruction: Result_Instruction,
}

impl Result_Execute {
    pub fn new() -> Self {
        Result_Execute {
            result_program: Result_Execute_program::Succes,
            dir_inst: 0,
            instruction: Palabra::new(&"00000000".to_string()).unwrap(),
            result_instruction: Result_Instruction::Palabra(
                Palabra::new(&"00000000".to_string()).unwrap(),
            ),
        }
    }
}

pub struct Registers_Cpu_Config {
    pub mode: Mode_Execute,
    pub rb: Palabra,
    pub rl: Palabra,
    pub rx: Palabra,
    pub sp: Palabra,
    pub pc: i32,
}
#[derive(Debug)]
pub struct Cpu {
    pub registers: Registros,
    ram: Arc<Mutex<Ram>>,
    pub external_interrupt: Arc<Mutex<External_interrupt>>,
    pub sender_dma: Sender<Dma_Config>,
    pub clock_interrupt: u32,
    pub dma_temp: Dma_Config,
    pub have_user_program: bool,
    pub result_last_program: Result_Execute,
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
            result_last_program: Result_Execute::new(),
            ram,
            external_interrupt,
            sender_dma,
        }
    }
    pub fn run(&mut self) {
        self.have_user_program = true;
        while self.have_user_program {
            self.step();
            println!("Registers: {:#?}", self.registers);
            sleep(Duration::from_millis(500));
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
        let result_vec = self.vector_interrupt();
        match result_vec {
            Ok(_) => (),
            Err(err) => {
                self.result_last_program.result_program = Result_Execute_program::Error;
                self.have_user_program = false;
            }
        }
    }

    fn vector_interrupt(&mut self) -> Result_op {
        let (
            overflow,
            underflow,
            dir_inv,
            inst_inv,
            io,
            clock,
            call_sys,
            cod_inte_inv,
            cod_callsys_inv,
        ) = {
            let ext = self.external_interrupt.lock().unwrap();
            (
                ext.int_overflow,
                ext.int_underflow,
                ext.int_dir_inv,
                ext.int_inst_inv,
                ext.int_io,
                ext.int_clock,
                ext.int_call_sys,
                ext.int_cod_inte_inv,
                ext.int_cod_callsys_inv,
            )
        };

        //Falta salvaguarda de estado para algunas instrucciones en el cambio de contexto

        if overflow {
            self.registers.psw.set_mode(1)?;
            self.registers.psw.pc = 8;
            return Ok(());
        }

        if underflow {
            self.registers.psw.set_mode(1)?;

            self.registers.psw.pc = 7;
            return Ok(());
        }
        if dir_inv {
            self.registers.psw.set_mode(1)?;

            self.registers.psw.pc = 6;
            return Ok(());
        }
        if inst_inv {
            self.registers.psw.set_mode(1)?;

            self.registers.psw.pc = 5;
            return Ok(());
        }
        if io {
            self.save_context()?;

            self.registers.psw.set_mode(1)?;

            self.registers.psw.pc = 4;
            return Ok(());
        }
        if clock {
            self.save_context()?;

            self.registers.psw.set_mode(1)?;

            self.registers.psw.pc = 3;
            return Ok(());
        }
        if call_sys {
            self.save_context()?;

            self.registers.psw.set_mode(1)?;

            self.registers.psw.pc = 2;
            return Ok(());
        }
        if cod_inte_inv {
            self.registers.psw.set_mode(1)?;
            self.registers.psw.pc = 1;
            return Ok(());
        }
        if cod_callsys_inv {
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
                self.result_last_program.result_instruction =
                    Result_Instruction::String(String::from("Error con la sincronización del bus"));
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
            if pos_mem_palabra < self.registers.rb || pos_mem_palabra >= self.registers.rx {
                self.result_last_program.result_instruction =
                    Result_Instruction::String(String::from("Fuera de los límites de memoria"));
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
                    self.result_last_program.result_instruction = Result_Instruction::String(
                        String::from(" Codigo de direccionamiento inválido"),
                    );
                    return Err(Errors {
                        msg: "Codigo de direccionamiento inválido".to_string(),
                        cod: Interrups::DirInv,
                    });
                }
            }
        } else {
            match self.registers.ir.dir {
                0 => self.dir_direct_store()?,
                2 => self.dir_indexed_store()?,
                _ => {
                    self.result_last_program.result_instruction = Result_Instruction::String(
                        String::from(" Codigo de direccionamiento inválido"),
                    );
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
            25 => self.psh(false)?,
            26 => self.pop(false)?,
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
                self.result_last_program.result_instruction =
                    Result_Instruction::String(String::from(format!("Llamada a sistema invalida")));
                match response_handle {
                    ContinueOrBreak::Break => {
                        self.have_user_program = false;
                        self.result_last_program.result_program = Result_Execute_program::Error;
                    }
                    ContinueOrBreak::Continue => {
                        self.restore_context();
                    }
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

                self.result_last_program.result_instruction = Result_Instruction::String(
                    String::from(format!("Codigo de interrupcion invalida")),
                );

                match response_handle {
                    ContinueOrBreak::Break => {
                        self.have_user_program = false;
                        self.result_last_program.result_program = Result_Execute_program::Error;
                    }
                    ContinueOrBreak::Continue => {
                        self.restore_context();
                    }
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

                if self.registers.ac.convert() == 1 {
                    self.result_last_program.result_instruction = Result_Instruction::String(
                        String::from(format!("Programa terminado correctamente")),
                    );
                } else {
                    self.result_last_program.result_instruction =
                        Result_Instruction::String(String::from(format!(
                            "Se ejecuto rutina Llamada el sistema codigo: {}",
                            self.registers.ac.convert()
                        )));
                }

                match response_handle {
                    ContinueOrBreak::Break => {
                        self.have_user_program = false;
                    }
                    ContinueOrBreak::Continue => {
                        self.restore_context();
                    }
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

                self.result_last_program.result_instruction =
                    Result_Instruction::String(String::from(format!("Llamada a sistema invalida")));
                match response_handle {
                    ContinueOrBreak::Break => {
                        self.have_user_program = false;
                        self.result_last_program.result_program = Result_Execute_program::Error;
                    }
                    ContinueOrBreak::Continue => {
                        self.restore_context();
                    }
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
                self.result_last_program.result_instruction =
                    Result_Instruction::String(String::from(format!("Termino I/O")));

                match response_handle {
                    ContinueOrBreak::Break => {
                        self.have_user_program = false;
                        self.result_last_program.result_program = Result_Execute_program::Error;
                    }
                    ContinueOrBreak::Continue => {
                        self.restore_context();
                    }
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
                self.result_last_program.result_instruction =
                    Result_Instruction::String(String::from(format!("Instrucción Inválida")));

                match response_handle {
                    ContinueOrBreak::Break => {
                        self.have_user_program = false;
                        self.result_last_program.result_program = Result_Execute_program::Error;
                    }
                    ContinueOrBreak::Continue => {
                        self.restore_context();
                    }
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

                self.result_last_program.result_instruction =
                    Result_Instruction::String(String::from(format!("Direccionamiento Inválido")));
                match response_handle {
                    ContinueOrBreak::Break => {
                        self.have_user_program = false;
                        self.result_last_program.result_program = Result_Execute_program::Error;
                    }
                    ContinueOrBreak::Continue => {
                        self.restore_context();
                    }
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

                self.result_last_program.result_instruction =
                    Result_Instruction::String(String::from(format!("Underflow")));
                match response_handle {
                    ContinueOrBreak::Break => {
                        self.have_user_program = false;
                        self.result_last_program.result_program = Result_Execute_program::Error;
                    }
                    ContinueOrBreak::Continue => {
                        self.restore_context();
                    }
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
                self.result_last_program.result_instruction =
                    Result_Instruction::String(String::from(format!("Overflow")));

                match response_handle {
                    ContinueOrBreak::Break => {
                        self.have_user_program = false;
                        self.result_last_program.result_program = Result_Execute_program::Error;
                    }
                    ContinueOrBreak::Continue => {
                        self.restore_context();
                    }
                }
            }
            _ => {
                self.result_last_program.result_instruction =
                    Result_Instruction::String(String::from("Instrucción invalida execute"));
                return Err(Errors {
                    msg: "Instrucción invalida execute".to_string(),
                    cod: Interrups::InstInv,
                });
            }
        }
        Ok(())
    }

    pub fn save_context(&mut self) -> Result_op {
        let ac_temp = self.registers.ac;
        //Guardo Ac en pila
        self.psh(true)?;

        //Guardo rb en la pila
        self.registers.ac = self.registers.rb;
        self.psh(true)?;

        //Guardo rl en la pila
        self.registers.ac = self.registers.rl;
        self.psh(true)?;

        //Guardo rx en la pila
        self.registers.ac = self.registers.rx;
        self.psh(true)?;

        //Guardo psw en la pila
        self.registers.ac = Palabra::new(&self.registers.psw.convert_to_palabra()).unwrap();
        self.psh(true)?;

        self.registers.ac = ac_temp;

        Ok(())
    }

    pub fn restore_context(&mut self) -> Result_op {
        //Restauro psw de la pila
        self.pop(true)?;
        self.registers
            .psw
            .convert_to_psw_by_palabra(self.registers.ac);

        //Restaura rx de la pila
        self.pop(true)?;
        self.registers.set_rx(self.registers.ac);

        //Restaura rl de la pila
        self.pop(true)?;
        self.registers.set_rl(self.registers.ac);
        //Restaura rb de la pila
        self.pop(true)?;
        self.registers.set_rb(self.registers.ac);
        //Restaura Ac de la pila
        self.pop(true)?;

        Ok(())
    }

    fn dir_direct(&mut self) -> Result_op {
        let dir_num: i32 = match self.registers.psw.modo_op {
            1 => self.registers.ir.value as i32,
            other => self.registers.rb.convert() + self.registers.ir.value as i32,
        };

        if self.registers.psw.modo_op == 1 {
            if dir_num > 2000 || dir_num < 0 {
                self.result_last_program.result_instruction =
                    Result_Instruction::String(String::from("Direccionamiento Invalido "));
                return Err(Errors {
                    msg: "Direccionamiento Invalido ".to_string(),
                    cod: Interrups::DirInv,
                });
            }
        } else {
            if dir_num >= self.registers.rx.convert() {
                self.result_last_program.result_instruction =
                    Result_Instruction::String(String::from("Direccionamiento Invalido "));
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
                self.result_last_program.result_instruction =
                    Result_Instruction::String(String::from("Direccionamiento Invalido "));
                return Err(Errors {
                    msg: "Direccionamiento Invalido ".to_string(),
                    cod: Interrups::DirInv,
                });
            }
        } else {
            if dir_num >= self.registers.rx.convert() {
                self.result_last_program.result_instruction =
                    Result_Instruction::String(String::from("Direccionamiento Invalido "));
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
                self.result_last_program.result_instruction =
                    Result_Instruction::String(String::from("Direccionamiento Invalido "));
                return Err(Errors {
                    msg: "Direccionamiento Invalido ".to_string(),
                    cod: Interrups::DirInv,
                });
            }
        } else {
            if dir_num >= self.registers.rx.convert() {
                self.result_last_program.result_instruction =
                    Result_Instruction::String(String::from("Direccionamiento Invalido "));
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
                self.result_last_program.result_instruction =
                    Result_Instruction::String(String::from("Direccionamiento Invalido "));
                return Err(Errors {
                    msg: "Direccionamiento Invalido ".to_string(),
                    cod: Interrups::DirInv,
                });
            }
        } else {
            if dir_num >= self.registers.rx.convert() {
                self.result_last_program.result_instruction =
                    Result_Instruction::String(String::from("Direccionamiento Invalido "));
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

    // pub fn pop_high_level(&mut self) -> Result<Palabra, Errors> {
    //     let new_sp = (self.registers.sp + Palabra::new("00000001").unwrap())?;

    //     let state_mem = self.ram.lock();
    //     let mut state_mem = convert_result(
    //         state_mem,
    //         "Error con la sincronización del bus".to_string(),
    //         Interrups::DirInv,
    //     )?;

    //     let value_stack = state_mem.readMemory(self.registers.sp.convert())?;
    //     self.registers.sp = new_sp;
    //     Ok(value_stack)
    // }

    pub fn sum(&mut self) -> Result_op {
        let pal = (self.registers.ac + self.registers.mdr);

        match pal {
            Ok(palabra) => {
                self.result_last_program.result_instruction = Result_Instruction::String(format!(
                    "Suma: Ac:{} + {} = {}",
                    self.registers.ac.convert(),
                    self.registers.mdr.convert(),
                    palabra.convert()
                ));
                Ok(self.registers.ac = palabra)
            }
            Err(err) => {
                self.registers.psw.set_codition(3)?;
                return Err(err);
            }
        }
    }

    pub fn rest(&mut self) -> Result_op {
        let pal = (self.registers.ac - self.registers.mdr);

        match pal {
            Ok(palabra) => {
                self.result_last_program.result_instruction = Result_Instruction::Palabra(palabra);
                Ok(self.registers.ac = palabra)
            }
            Err(err) => {
                self.registers.psw.set_codition(3)?;
                return Err(err);
            }
        }
    }
    pub fn mult(&mut self) -> Result_op {
        let pal = (self.registers.ac * self.registers.mdr);

        match pal {
            Ok(palabra) => {
                self.result_last_program.result_instruction = Result_Instruction::Palabra(palabra);
                Ok(self.registers.ac = palabra)
            }
            Err(err) => {
                self.registers.psw.set_codition(3)?;
                return Err(err);
            }
        }
    }
    pub fn divi(&mut self) -> Result_op {
        let pal = (self.registers.ac / self.registers.mdr);

        match pal {
            Ok(palabra) => {
                self.result_last_program.result_instruction = Result_Instruction::Palabra(palabra);
                Ok(self.registers.ac = palabra)
            }
            Err(err) => {
                self.registers.psw.set_codition(3)?;
                return Err(err);
            }
        }
    }

    pub fn load(&mut self) -> Result_op {
        self.registers.ac = self.registers.mdr;
        self.result_last_program.result_instruction = Result_Instruction::String(String::from(
            format!("Se movio {} -> [AC]", self.registers.mdr.convert()),
        ));
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
        self.result_last_program.result_instruction = Result_Instruction::String(String::from(
            format!("Se movio [AC] -> RAM[{}]", self.registers.mar.convert()),
        ));
        Ok(())
    }

    pub fn load_rx(&mut self) -> Result_op {
        self.registers.ac = self.registers.rx;
        self.result_last_program.result_instruction =
            Result_Instruction::Palabra(self.registers.ac);
        Ok(())
    }
    pub fn store_rx(&mut self) -> Result_op {
        self.registers.rx = self.registers.ac;
        self.result_last_program.result_instruction =
            Result_Instruction::Palabra(self.registers.rx);
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

        self.result_last_program.result_instruction =
            Result_Instruction::String(String::from(format!(
                "Comparación AC: {} con MDR: {}",
                self.registers.ac.convert(),
                self.registers.mdr.convert()
            )));
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

            self.result_last_program.result_instruction = Result_Instruction::String(String::from(
                format!("Salto a la dirección {:?}", self.registers.mdr.convert()),
            ));
        }

        self.result_last_program.result_instruction = Result_Instruction::String(String::from(
            format!("No Salto a la dirección {:?}", self.registers.mdr.convert()),
        ));

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
            self.result_last_program.result_instruction = Result_Instruction::String(String::from(
                format!("Salto a la dirección {}", self.registers.mdr.convert()),
            ));
        }
        self.result_last_program.result_instruction =
            Result_Instruction::String(String::from(format!(
                " No Salto a la dirección {:?}",
                self.registers.mdr.convert()
            )));
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
            self.result_last_program.result_instruction = Result_Instruction::String(String::from(
                format!("Salto a la dirección {:?}", self.registers.mdr.convert()),
            ));
        }
        self.result_last_program.result_instruction = Result_Instruction::String(String::from(
            format!("No Salto a la dirección {:?}", self.registers.mdr.convert()),
        ));
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
            self.result_last_program.result_instruction = Result_Instruction::String(String::from(
                format!("Salto a la dirección {:?}", self.registers.mdr.convert()),
            ));
        }

        self.result_last_program.result_instruction = Result_Instruction::String(String::from(
            format!("No Salto a la dirección {:?}", self.registers.mdr.convert()),
        ));
        Ok(())
    }

    pub fn svc(&mut self) -> Result_op {
        self.result_last_program.result_instruction = Result_Instruction::String(String::from(
            format!("Llamada al sistema {:?}", self.registers.ac.convert()),
        ));
        Err(Errors {
            msg: "Llamada al sistema".to_string(),
            cod: Interrups::CallSys,
        })
    }

    pub fn psh(&mut self, is_save_context: bool) -> Result_op {
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
        if !is_save_context {
            self.result_last_program.result_instruction = Result_Instruction::String(String::from(
                format!("Push de ac  {:?}", self.registers.ac.convert()),
            ));
        }

        Ok(())
    }

    pub fn pop(&mut self, is_save_context: bool) -> Result_op {
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
        if !is_save_context {
            self.result_last_program.result_instruction = Result_Instruction::String(String::from(
                format!("Pop de ac {:?}", self.registers.ac.convert()),
            ));
        }

        Ok(())
    }

    pub fn retrn(&mut self) -> Result_op {
        self.pop(false)?;

        self.registers.psw.set_pc(self.registers.ac.convert())?;
        self.result_last_program.result_instruction =
            Result_Instruction::String(String::from(format!(
                "Retorno de subrutina hacia  ac {:?}",
                self.registers.ac.convert()
            )));
        Ok(())
    }

    pub fn hab(&mut self) -> Result_op {
        if self.registers.psw.modo_op == 0 {
            return Err(Errors {
                msg: "Falta de privilegios, se necesita modo kernel".to_string(),
                cod: Interrups::InstInv,
            });
        }

        self.registers.psw.set_inte(1)?;
        self.result_last_program.result_instruction =
            Result_Instruction::String(String::from(format!("Se habilitan interrupciones")));

        Ok(())
    }
    pub fn dhab(&mut self) -> Result_op {
        if self.registers.psw.modo_op == 0 {
            return Err(Errors {
                msg: "Falta de privilegios, se necesita modo kernel".to_string(),
                cod: Interrups::InstInv,
            });
        }

        self.registers.psw.set_inte(0)?;
        self.result_last_program.result_instruction = Result_Instruction::String(String::from(
            format!("Se desahabilitan las interrupciones"),
        ));
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

        self.result_last_program.result_instruction = Result_Instruction::String(String::from(
            format!("Se cambio las interrupciones de reloj a {:?}", data),
        ));
        Ok(())
    }

    pub fn chmod(&mut self) -> Result_op {
        if self.registers.psw.modo_op == 0 {
            return Err(Errors {
                msg: "Falta de privilegios, se necesita modo kernel".to_string(),
                cod: Interrups::InstInv,
            });
        }

        self.registers.psw.set_mode(0)?;

        // self.result_last_program.result_instruction =
        //     Result_Instruction::String(String::from(format!("Se cambió de modo a modo usuario")));

        Ok(())
    }

    pub fn load_rb(&mut self) -> Result_op {
        self.registers.ac = self.registers.rb;

        self.result_last_program.result_instruction = Result_Instruction::String(String::from(
            format!("Cargo rb en Ac {:?}", self.registers.ac.convert()),
        ));
        Ok(())
    }

    pub fn store_rb(&mut self) -> Result_op {
        self.registers.set_rb(self.registers.ac)?;
        self.result_last_program.result_instruction = Result_Instruction::String(String::from(
            format!("Cargo Ac en Rb {:?}", self.registers.ac.convert()),
        ));
        Ok(())
    }
    pub fn load_rl(&mut self) -> Result_op {
        self.registers.ac = self.registers.rl;
        self.result_last_program.result_instruction = Result_Instruction::String(String::from(
            format!("Cargo Rl en Ac {:?}", self.registers.ac.convert()),
        ));
        Ok(())
    }
    pub fn store_rl(&mut self) -> Result_op {
        self.registers.set_rl(self.registers.ac)?;
        self.result_last_program.result_instruction = Result_Instruction::String(String::from(
            format!("Cargo Ac en Rl {:?}", self.registers.ac.convert()),
        ));
        Ok(())
    }
    pub fn load_sp(&mut self) -> Result_op {
        self.registers.ac = self.registers.sp;
        self.result_last_program.result_instruction = Result_Instruction::String(String::from(
            format!("Cargo Sp en Ac {:?}", self.registers.ac.convert()),
        ));
        Ok(())
    }

    pub fn store_sp(&mut self) -> Result_op {
        self.registers.set_sp(self.registers.ac)?;
        self.result_last_program.result_instruction = Result_Instruction::String(String::from(
            format!("Cargo Ac en Sp {:?}", self.registers.ac.convert()),
        ));
        Ok(())
    }

    //Esto permite que se hagan saltos indirecto, es decir, cuando el modo de direccionamiento sea distinto a inmediato. Lo que va a suceder es que la dirección se comporta como un puntero
    pub fn j(&mut self) -> Result_op {
        self.registers.psw.set_pc(self.registers.mdr.convert())?;
        self.result_last_program.result_instruction = Result_Instruction::String(String::from(
            format!("Salto indirecto a {:?}", self.registers.mdr.convert()),
        ));
        Ok(())
    }

    pub fn sdmap(&mut self) -> Result_op {
        self.dma_temp.pista_acceder = self.registers.mdr.convert() as i8;
        self.result_last_program.result_instruction =
            Result_Instruction::String(String::from(format!(
                " Se seteo la pista en el dma {:?}",
                self.registers.mdr.convert()
            )));
        Ok(())
    }
    pub fn sdmac(&mut self) -> Result_op {
        self.dma_temp.cil_acceder = self.registers.mdr.convert() as i8;
        self.result_last_program.result_instruction =
            Result_Instruction::String(String::from(format!(
                " Se seteo el cilindro en el dma {:?}",
                self.registers.mdr.convert()
            )));
        Ok(())
    }
    pub fn sdmas(&mut self) -> Result_op {
        self.dma_temp.sector_acceder = self.registers.mdr.convert() as i8;
        self.result_last_program.result_instruction =
            Result_Instruction::String(String::from(format!(
                " Se seteo el sector en el dma {:?}",
                self.registers.mdr.convert()
            )));
        Ok(())
    }
    pub fn sdmaio(&mut self) -> Result_op {
        self.dma_temp.modo = self.registers.mdr.convert() as i8;
        self.result_last_program.result_instruction =
            Result_Instruction::String(String::from(format!(
                " Se seteo el modo en el dma {:?}",
                self.registers.mdr.convert()
            )));
        Ok(())
    }
    pub fn sdmam(&mut self) -> Result_op {
        self.dma_temp.pos_men = self.registers.mdr.convert();
        self.result_last_program.result_instruction =
            Result_Instruction::String(String::from(format!(
                " Se seteo la posicion de memoria en el dma {:?}",
                self.registers.mdr.convert()
            )));
        Ok(())
    }
    pub fn sdmaon(&mut self) -> Result_op {
        self.result_last_program.result_instruction =
            Result_Instruction::String(String::from(format!(" Se inicio la operación dma ")));
        convert_result(
            self.sender_dma.send(self.dma_temp),
            "Error al enviar orden dma".to_string(),
            Interrups::InstInv,
        )?;
        Ok(())
    }
}

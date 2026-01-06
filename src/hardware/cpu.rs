use crate::hardware::{architecture::Palabra, interrupts::Interrups};
use crate::utils::Errors;
#[derive(Debug)]
pub struct ExternalInterrup {
    pub int_io: bool,
    pub int_reloj: bool,
}

impl ExternalInterrup {
    pub fn new() -> Self {
        ExternalInterrup {
            int_io: false,
            int_reloj: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pws {
    pub cod_codicion: i8,
    pub modo_op: i8,
    pub inte: i8, //Interupciones
    pub pc: i32,
}

impl Pws {
    fn new() -> Self {
        Pws {
            cod_codicion: 0,
            modo_op: 0,
            inte: 1,
            pc: 0,
        }
    }

    pub fn set_codition(&mut self, val: i8) -> Result<(), Errors> {
        if val > 3 || val < 0 {
            return Err(Errors {
                msg: String::from("cod condicion invalido"),
                cod: Interrups::InstInv,
            });
        }

        self.cod_codicion = val;

        Ok(())
    }
    pub fn set_mode(&mut self, val: i8) -> Result<(), Errors> {
        if val > 1 || val < 0 {
            return Err(Errors {
                msg: String::from("cod modo invalido"),
                cod: Interrups::InstInv,
            });
        }

        self.modo_op = val;

        Ok(())
    }
    pub fn set_inte(&mut self, val: i8) -> Result<(), Errors> {
        if val > 1 || val < 0 {
            return Err(Errors {
                msg: String::from("allow inte invalido"),
                cod: Interrups::InstInv,
            });
        }

        self.inte = val;

        Ok(())
    }
    pub fn set_pc(&mut self, val: i32) -> Result<(), Errors> {
        if val > 99999 || val < 0 {
            return Err(Errors {
                msg: String::from("Dir Pc invalido"),
                cod: Interrups::InstInv,
            });
        }

        self.pc = val;

        Ok(())
    }

    pub fn convert_to_palabra(&self) -> String {
        format!(
            "{}{}{}{:05}",
            self.cod_codicion, self.modo_op, self.inte, self.pc
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Registros {
    pub mar: Palabra,
    pub mdr: Palabra,
    pub ir: Palabra,
    pub rb: Palabra,
    pub rl: Palabra,
    pub rx: Palabra,
    pub sp: Palabra,
    pub psw: Pws,
    pub ac: Palabra,
}

impl Registros {
    pub fn new() -> Self {
        Registros {
            mar: Palabra::new("00000000").unwrap(),
            mdr: Palabra::new("00000000").unwrap(),
            ir: Palabra::new("00000000").unwrap(),
            rb: Palabra::new("00000000").unwrap(),
            rl: Palabra::new("00000000").unwrap(),
            rx: Palabra::new("00000000").unwrap(),
            sp: Palabra::new("00000000").unwrap(),
            psw: Pws::new(),
            ac: Palabra::new("00000000").unwrap(),
        }
    }

    pub fn set_mar(&mut self, pal: Palabra) -> Result<(), Errors> {
        if pal.convert() > 2000 {
            let err = Errors {
                msg: String::from("Dirección de memoria invalida"),
                cod: Interrups::Overflow,
            };

            return Err(err);
        }

        if pal.convert() < 0 {
            let err = Errors {
                msg: String::from(" Dirección de memoria invalida"),
                cod: Interrups::Underflow,
            };

            return Err(err);
        }

        self.mar = pal;
        Ok(())
    }
    pub fn set_mdr(&mut self, pal: Palabra) {
        self.mdr = pal;
    }
    pub fn set_ir(&mut self, pal: Palabra) {
        self.ir = pal;
    }

    pub fn set_rb(&mut self, pal: Palabra) -> Result<(), Errors> {
        if pal.convert() > 2000 {
            let err = Errors {
                msg: String::from("Dirección de memoria invalida"),
                cod: Interrups::Overflow,
            };

            return Err(err);
        }
        if pal.convert() < 0 {
            let err = Errors {
                msg: String::from(" Dirección de memoria invalida"),
                cod: Interrups::Underflow,
            };

            return Err(err);
        }

        self.rb = pal;

        Ok(())
    }
    pub fn set_rl(&mut self, pal: Palabra) -> Result<(), Errors> {
        if pal.convert() > 2000 {
            let err = Errors {
                msg: String::from("Dirección de memoria invalida"),
                cod: Interrups::Overflow,
            };

            return Err(err);
        }
        if pal.convert() < 0 {
            let err = Errors {
                msg: String::from(" Dirección de memoria invalida"),
                cod: Interrups::Underflow,
            };

            return Err(err);
        }

        self.rl = pal;
        Ok(())
    }
    pub fn set_rx(&mut self, pal: Palabra) -> Result<(), Errors> {
        if pal.convert() > 2000 {
            let err = Errors {
                msg: String::from("Dirección de memoria invalida"),
                cod: Interrups::Overflow,
            };

            return Err(err);
        }
        if pal.convert() < 0 {
            let err = Errors {
                msg: String::from(" Dirección de memoria invalida"),
                cod: Interrups::Underflow,
            };

            return Err(err);
        }

        self.rx = pal;
        Ok(())
    }
    pub fn set_sp(&mut self, pal: Palabra) -> Result<(), Errors> {
        if pal.convert() > 2000 {
            let err = Errors {
                msg: String::from("Dirección de memoria invalida"),
                cod: Interrups::Overflow,
            };

            return Err(err);
        }
        if pal.convert() < 0 {
            let err = Errors {
                msg: String::from(" Dirección de memoria invalida"),
                cod: Interrups::Underflow,
            };

            return Err(err);
        }

        self.sp = pal;

        Ok(())
    }
}

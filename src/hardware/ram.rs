use crate::{
    hardware::{architecture::Palabra, interrupts::Interrups, registers::Registros},
    utils::Errors,
};
#[derive(Debug, PartialEq)]
pub struct Ram {
    ram: [Palabra; 2001],
}

impl Ram {
    pub fn new() -> Self {
        Ram {
            ram: [Palabra::new(&"00000000".to_string()).unwrap(); 2001],
        }
    }

    pub fn readMemory(&self, position_read: i32) -> Result<Palabra, Errors> {
        if position_read > 2001 || position_read < 0 {
            return Err(Errors {
                msg: String::from("Dirección a leer invalida"),
                cod: Interrups::DirInv,
            });
        }

        Ok(self.ram[position_read as usize])
    }

    pub fn writeMemory(&mut self, position_write: i32, pal: Palabra) -> Result<(), Errors> {
        if position_write > 2001 || position_write < 0 {
            return Err(Errors {
                msg: String::from("Dirección a leer invalida"),
                cod: Interrups::DirInv,
            });
        }

        self.ram[position_write as usize] = pal;
        Ok(())
    }
}

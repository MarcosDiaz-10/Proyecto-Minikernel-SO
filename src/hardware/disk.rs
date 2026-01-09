use crate::hardware::interrupts::Interrups;
use crate::utils::{Errors, Result_op};
pub type SectorData = [u8; 9];
pub struct Disk {
    disk: [[[SectorData; 101]; 11]; 11],
}

impl Disk {
    pub fn new() -> Self {
        Disk {
            disk: [[[[48; 9]; 101]; 11]; 11],
        }
    }

    pub fn read(&self, cil: i8, pista: i8, sec: i8) -> Result<String, Errors> {
        if (cil > 10 || cil < 0) || (pista > 10 || pista < 0) || (sec > 100 || sec < 0) {
            return Err(Errors {
                msg: String::from("Error al leer del disco"),
                cod: Interrups::EndIO,
            });
        }
        let data = &self.disk[cil as usize][pista as usize][sec as usize][0..8];

        Ok(String::from_utf8_lossy(data).to_string())
    }

    pub fn write(&mut self, data: String, cil: i8, pista: i8, sec: i8) -> Result_op {
        if (cil > 10 || cil < 0) || (pista > 10 || pista < 0) || (sec > 100 || sec < 0) {
            return Err(Errors {
                msg: String::from("Error al escribir del disco"),
                cod: Interrups::EndIO,
            });
        }

        if data.len() != 9 {
            return Err(Errors {
                msg: String::from("Error al escribir del disco"),
                cod: Interrups::EndIO,
            });
        }

        let mut bloque = [48u8; 9];

        for (i, &b) in data.as_bytes().iter().enumerate() {
            bloque[i] = b;
        }

        self.disk[cil as usize][pista as usize][sec as usize] = bloque;

        Ok(())
    }
}

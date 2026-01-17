use std::{
    ops::{Add, Div, Mul, Sub},
    process::Output,
};

use crate::{
    hardware::interrupts::Interrups,
    utils::{Errors, convert_option_result, convert_to_string_format_pal},
};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Palabra {
    pub palabra: u32,
}

impl Palabra {
    pub fn new(num: &str) -> Option<Self> {
        if num.len() != 8 {
            return None;
        }

        let parse_number = num.trim().parse::<u32>().ok();

        match parse_number {
            Some(num) => Some(Palabra { palabra: num }),
            None => None,
        }
    }

    pub fn convert(&self) -> i32 {
        let sig = self.palabra / 10000000;
        match sig {
            0 => (self.palabra) as i32,
            1 => -1 * (self.palabra - 10000000) as i32,
            _ => 1,
        }
    }

    pub fn convert_to_string_disk(&self) -> String {
        format!("{:08}F", self.palabra)
    }

    pub fn convert_to_disk_palabra(s: &str) -> Option<Self> {
        if s.len() != 9 {
            return None;
        }

        let trimmed = &s[0..8];
        let parse_number = trimmed.trim().parse::<u32>().ok();

        match parse_number {
            Some(num) => Some(Palabra { palabra: num }),
            None => None,
        }
    }
}

impl Add for Palabra {
    type Output = Result<Self, Errors>;

    fn add(self, other: Self) -> Result<Self, Errors> {
        let sum = self.convert() + other.convert();

        if sum > 9999999 {
            return Err(Errors {
                msg: "Overflow al realizar suma".to_string(),
                cod: Interrups::Overflow,
            });
        } else if sum < -9999999 {
            return Err(Errors {
                msg: "Underflow al realizar suma".to_string(),
                cod: Interrups::Underflow,
            });
        }

        let string_pal = convert_to_string_format_pal(sum);
        let result = convert_option_result(
            Self::new(&string_pal),
            format!(
                "Error al realizar la instrucción suma, codInterrup {:?}",
                Interrups::InstInv
            )
            .to_string(),
            Interrups::InstInv,
        )?;
        Ok(result)
    }
}

impl Sub for Palabra {
    type Output = Result<Self, Errors>;

    fn sub(self, other: Self) -> Result<Self, Errors> {
        let rest = self.convert() - other.convert();

        if rest > 9999999 {
            return Err(Errors {
                msg: "Overflow al realizar resta".to_string(),
                cod: Interrups::Overflow,
            });
        } else if rest < -9999999 {
            return Err(Errors {
                msg: "Underflow al realizar resta".to_string(),
                cod: Interrups::Underflow,
            });
        }

        let string_pal = convert_to_string_format_pal(rest);
        let result = convert_option_result(
            Self::new(&string_pal),
            format!(
                "Error al realizar la instrucción resta, codInterrup {:?}",
                Interrups::InstInv
            )
            .to_string(),
            Interrups::InstInv,
        )?;
        Ok(result)
    }
}

impl Mul for Palabra {
    type Output = Result<Self, Errors>;
    fn mul(self, other: Self) -> Self::Output {
        let mult = self.convert() * other.convert();

        if mult > 9999999 {
            return Err(Errors {
                msg: "Overflow al realizar Multiplicacion".to_string(),
                cod: Interrups::Overflow,
            });
        } else if mult < -9999999 {
            return Err(Errors {
                msg: "Underflow al realizar Multiplicacion".to_string(),
                cod: Interrups::Underflow,
            });
        }

        let string_pal = convert_to_string_format_pal(mult);
        let result = convert_option_result(
            Self::new(&string_pal),
            format!(
                "Error al realizar la instrucción Multiplicacion, codInterrup {:?}",
                Interrups::InstInv
            )
            .to_string(),
            Interrups::InstInv,
        )?;
        Ok(result)
    }
}

impl Div for Palabra {
    type Output = Result<Self, Errors>;
    fn div(self, other: Self) -> Self::Output {
        let divisor = other.convert();
        if divisor == 0 {
            return Err(Errors {
                msg: "Instruccion invalida, division por 0".to_string(),
                cod: Interrups::InstInv,
            });
        }

        let div = self.convert() * other.convert();

        if div > 9999999 {
            return Err(Errors {
                msg: "Overflow al realizar division".to_string(),
                cod: Interrups::Overflow,
            });
        } else if div < -9999999 {
            return Err(Errors {
                msg: "Underflow al realizar division".to_string(),
                cod: Interrups::Underflow,
            });
        }

        let string_pal = convert_to_string_format_pal(div);
        let result = convert_option_result(
            Self::new(&string_pal),
            format!(
                "Error al realizar la instrucción division, codInterrup {:?}",
                Interrups::InstInv
            )
            .to_string(),
            Interrups::InstInv,
        )?;
        Ok(result)
    }
}

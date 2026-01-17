use std::{
    fs::File,
    io::{BufRead, BufReader, Error},
    os::unix::process,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::{
    Programs,
    hardware::{architecture::Palabra, ram::Ram},
};

use super::Interrups;
#[derive(Debug)]
pub struct Errors {
    pub msg: String,
    pub cod: Interrups,
}

pub type Result_op = Result<(), Errors>;
#[derive(Debug)]
pub enum ContinueOrBreak {
    Continue,
    Break,
}
//Transforma el result de una función en otro, por otro que devuelva mi error definido manualmente
pub fn convert_result<T, E>(
    result_extern: Result<T, E>,
    msg: String,
    cod: Interrups,
) -> Result<T, Errors> {
    let res = match result_extern {
        Ok(val) => Ok(val),
        Err(_) => Err(Errors { msg, cod }),
    };

    res
}
//Transforma un Option<T> en un Result<T,Erros>, para manejar el error de los None
pub fn convert_option_result<T>(
    option: Option<T>,
    msg: String,
    cod: Interrups,
) -> Result<T, Errors> {
    match option {
        Some(val) => Ok(val),
        None => Err(Errors { msg, cod }),
    }
}
//Transforma de i32 a String Palabra
pub fn convert_to_string_format_pal(pal: i32) -> String {
    if pal < 0 {
        return format!("1{:07}", pal.abs());
    } else {
        return format!("{:08}", pal);
    }
}

pub fn load_program_in_ram(
    path: &str,
    table_procces: &mut Vec<Programs>,
    ram: Arc<Mutex<Ram>>,
    position_to_load: i32,
) -> Result<(), Errors> {
    let file = File::open(path);

    let file = match file {
        Ok(file) => Ok(file),
        Err(e) => Err(Errors {
            msg: format!("Error al leer el archivo {}", e).to_string(),
            cod: Interrups::EndIO,
        }),
    }?;

    let reader = BufReader::new(file);

    let mut process: Programs = Programs::new();

    for (i, line) in reader.lines().enumerate() {
        let l = line;
        let l = match l {
            Ok(l) => Ok(l),
            Err(e) => Err(Errors {
                msg: format!("Error al leer el archivo {}", e).to_string(),
                cod: Interrups::EndIO,
            }),
        }?;

        match i {
            0 => {
                for (j, sp) in l.split_whitespace().enumerate() {
                    match j {
                        1 => process.pos_start_program = sp.parse::<i32>().unwrap(),
                        _ => (),
                    }
                }
            }
            1 => {
                for (j, sp) in l.split_whitespace().enumerate() {
                    match j {
                        1 => {
                            process.num_instruccions_with_pila =
                                (sp.parse::<i32>().unwrap() * 2) + 1
                        }
                        _ => (),
                    }
                }

                {
                    let ram_s = ram.lock().unwrap();
                    let position_end = position_to_load + process.num_instruccions_with_pila;
                    let condition = ram_s.is_empty(position_to_load, position_end)?;

                    if !condition {
                        return Err(Errors {
                            msg: format!("Error al Cargar el archivo, posicion ocupada")
                                .to_string(),
                            cod: Interrups::EndIO,
                        });
                    };
                }
            }
            2 => {
                for (j, sp) in l.split_whitespace().enumerate() {
                    match j {
                        1 => process.name = sp.to_string(),
                        _ => (),
                    }
                }
            }
            _ => {
                for (j, sp) in l.split_whitespace().enumerate() {
                    match j {
                        0 => {
                            process.pos_start_mem = position_to_load;
                            let mut ram = ram.lock().unwrap();
                            let palabra = Palabra::new(sp).unwrap();
                            ram.writeMemory(position_to_load + (i as i32 - 3), palabra)?;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    table_procces.push(process);
    Ok(())
}

pub fn linear_search_program<'a>(
    table_process: &'a Vec<Programs>,
    name_program: &String,
) -> Result<&'a Programs, Errors> {
    for program in table_process {
        if program.name == *name_program {
            return Ok(program);
        }
    }
    Err(Errors {
        msg: format!(
            "No se encontro programa el programa {}, en la tabla de procesos",
            name_program
        )
        .to_string(),
        cod: Interrups::InstInv,
    })
}

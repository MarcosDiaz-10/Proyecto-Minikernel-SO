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

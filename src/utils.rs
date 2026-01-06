use super::Interrups;
#[derive(Debug)]
pub struct Errors {
    pub msg: String,
    pub cod: Interrups,
}

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

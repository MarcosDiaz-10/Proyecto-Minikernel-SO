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

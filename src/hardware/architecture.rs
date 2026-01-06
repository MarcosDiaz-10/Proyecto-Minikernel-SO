#[derive(Debug, Clone, Copy, PartialEq)]
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
        println!("{sig}");
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

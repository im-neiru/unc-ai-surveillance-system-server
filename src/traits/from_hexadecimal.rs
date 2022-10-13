pub trait FromHexadecimal {
    fn from_hexadecimal(hex: &str) -> Result<Self, HexParseErr> where Self: Sized;
}

macro_rules! int_of {
    ($value:expr) => {
        match ($value) {
            '0' => Ok(0),
            '1' => Ok(1),
            '2' => Ok(2),
            '3' => Ok(3),
            '4' => Ok(4),
            '5' => Ok(5),
            '6' => Ok(6),
            '7' => Ok(7),
            '8' => Ok(8),
            '9' => Ok(9),
            'A' | 'a' => Ok(10),
            'B' | 'b' => Ok(11),
            'C' | 'c' => Ok(12),
            'D' | 'd' => Ok(13),
            'E' | 'e' => Ok(14),
            'F' | 'f' => Ok(15),
            _ => Err(HexParseErr (HexParseErrReason::InvalidCharacter($value)))
        }
    };
}

impl FromHexadecimal for u128 {
    fn from_hexadecimal(hex: &str) -> Result<u128, HexParseErr> {
        use seq_macro::seq;

        let mut chars = hex.chars();
        let mut int = int_of!(chars.next()
            .ok_or(HexParseErr(HexParseErrReason::EmptyString))?)?;

        seq!(N in 0..32 {
            if let Some(c) = chars.next() {
                int = int << 4 | int_of!(c)?;
            } else {
                return Ok(int)
            }
        });

        Ok(int)
    }
}

pub struct HexParseErr(HexParseErrReason);

impl std::fmt::Display for HexParseErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            HexParseErrReason::EmptyString =>
                f.write_str("Empty string"),
            HexParseErrReason::InvalidCharacter(chr) => 
                write!(f, "'{}' is not a hexadecimal base", chr),
        }
    }
}

impl std::fmt::Debug for HexParseErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self, f)
    }
}

impl std::error::Error for HexParseErr { }

enum HexParseErrReason {
    EmptyString,
    InvalidCharacter(char),
}
pub trait FromHexadecimal {
    fn from_hexadecimal(hex: &str) -> Result<Self, HexParseErr> where Self: Sized;
}

pub struct HexParseErr;
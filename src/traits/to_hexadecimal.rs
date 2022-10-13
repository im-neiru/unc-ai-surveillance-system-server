use seq_macro::seq;

pub trait ToHexadecimal {
    fn to_hexadecimal(&self) -> String;
}

macro_rules! hex_of {
    ($value:expr) => {
        if ($value) <  0x0a {
            (($value) as u8 + 0x30) as char
        } else {
            (($value) as u8 + 0x37) as char
        }
    };
}

impl ToHexadecimal for u128 {

    fn to_hexadecimal(&self) -> String {
        let mut result = String::with_capacity(32);

        seq!(N in 1..=32 {
            result.push(hex_of!((self >> (128 - N * 4)) & 0x0f));
        });
        
        result
    }
}
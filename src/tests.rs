use crate::traits::ToHexadecimal;
use crate::traits::FromHexadecimal;

#[test]
fn test_hexadecimal() {
    assert!(0x60A344u128.to_hexadecimal() == "0000000000000000000000000060A344",
        "Erroneous conversion to hexadecimal");

    assert!(u128::from_hexadecimal("0000000000000000000000000060A344").unwrap() == 0x60A344,
    "Erroneous conversion from hexadecimal");

    assert!(u128::from_hexadecimal("A00000060A344").unwrap() == 0xA00000060A344,
    "Erroneous conversion from hexadecimal");
}
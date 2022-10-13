use crate::traits::ToHexadecimal;

#[test]
fn test_hexadecimal() {
    assert!(0x60A344u128.to_hexadecimal() == "0000000000000000000000000060A344",
        "Erroneous hexadecimal conversion");
}
#[derive(Clone, Copy)]
#[derive(PartialEq, Eq)]
pub struct DeviceSignature(SignitureBits);

#[repr(C)]
#[derive(Clone, Copy, Eq)]
union SignitureBits {
    integer: u128,
    bytes: [u8; 16]
}

impl PartialEq for SignitureBits {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            <u128 as PartialEq>::eq(&self.integer, &other.integer)
        }
    }

    fn ne(&self, other: &Self) -> bool {
        unsafe {
            <u128 as PartialEq>::ne(&self.integer, &other.integer)
        }
    }
}

impl core::fmt::Debug for DeviceSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#032X}", unsafe { self.0.integer })
    }
}

impl std::fmt::Display for DeviceSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#032X}", unsafe { self.0.integer })
    }
}
pub struct PasswordHash([u8; 64]);

impl From<[u8; 64]> for PasswordHash {
    #[inline]
    fn from(arr: [u8; 64]) -> Self {
        Self(arr)
    }
}
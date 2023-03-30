use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;
use diesel::deserialize::FromSql;
use diesel::pg::Pg;
use diesel::serialize::ToSql;
use diesel::sql_types::Binary;
use diesel::{AsExpression, FromSqlRow};
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, AsExpression, FromSqlRow)]
#[diesel(sql_type = diesel::sql_types::Binary)]
pub struct PasswordHash([u8; 64]);

impl From<[u8; 64]> for PasswordHash {
    #[inline]
    fn from(arr: [u8; 64]) -> Self {
        Self(arr)
    }
}

impl Display for PasswordHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&STANDARD_NO_PAD.encode(self.0))
    }
}

impl<'a> ToSql<Binary, Pg> for PasswordHash
where
    &'a [u8]: ToSql<Binary, Pg>,
{
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        <[u8] as ToSql<Binary, Pg>>::to_sql(self.0.as_ref(), out)
    }
}

impl FromSql<Binary, Pg> for PasswordHash
where
    Vec<u8>: FromSql<Binary, Pg>,
{
    fn from_sql(bytes: diesel::backend::RawValue<'_, Pg>) -> diesel::deserialize::Result<Self> {
        if let Ok(arr) = <Vec<u8> as FromSql<Binary, Pg>>::from_sql(bytes)?.try_into() {
            return Ok(Self(arr));
        }

        Err("Cannot convert to PasswordHash".into())
    }
}

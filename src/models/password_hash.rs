use std::fmt::Display;

use diesel::AsExpression;
use diesel::pg::Pg;
use diesel::serialize::ToSql;
use diesel::deserialize::FromSql;
use diesel::sql_types::Binary;

#[derive(Debug, Clone)]
#[derive(AsExpression)]
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
        self.to_string().fmt(f)
    }
}

impl<'a> ToSql<Binary, Pg> for PasswordHash where &'a [u8]: ToSql<Binary, Pg> {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
        <[u8] as ToSql<Binary, Pg>>::to_sql(self.0.as_ref(), out)
    }
}

impl FromSql<Binary, Pg> for PasswordHash where Vec<u8>: FromSql<Binary, Pg> {
    fn from_sql(bytes: diesel::backend::RawValue<'_, Pg>) -> diesel::deserialize::Result<Self> {
        if let Ok(arr) = <Vec<u8> as FromSql<Binary, Pg>>::from_sql(bytes)?.try_into() {
            return Ok(Self(arr));
        }

        Err("Cannot convert to PasswordHash".into())
    }
}
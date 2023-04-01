use diesel::deserialize::FromSql;
use diesel::pg::Pg;
use diesel::serialize::ToSql;
use diesel::sql_types::SmallInt;
use diesel::{AsExpression, FromSqlRow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, AsExpression, FromSqlRow)]
#[diesel(sql_type = SmallInt)]
pub enum UserRole {
    SecurityGuard = 1,
    SecurityHead = 2,
    SystemAdmin = 3,
}

impl Serialize for UserRole {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(match *self {
            UserRole::SecurityGuard => 1,
            UserRole::SecurityHead => 2,
            UserRole::SystemAdmin => 3,
        })
    }
}

impl<'de> Deserialize<'de> for UserRole {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u32::deserialize(deserializer)?;
        match value {
            0 => Ok(Self::SecurityGuard),
            1 => Ok(Self::SecurityHead),
            2 => Ok(Self::SystemAdmin),
            _ => Err(serde::de::Error::custom(format!(
                "invalid value for User role: {value}"
            ))),
        }
    }
}

const NUMERIC_VALUES: [i16; 3] = [1, 2, 3];

impl ToSql<SmallInt, Pg> for UserRole
where
    i16: ToSql<SmallInt, Pg>,
{
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        <i16 as ToSql<SmallInt, Pg>>::to_sql(
            match self {
                UserRole::SecurityGuard => &NUMERIC_VALUES[0],
                UserRole::SecurityHead => &NUMERIC_VALUES[1],
                UserRole::SystemAdmin => &NUMERIC_VALUES[2],
            },
            out,
        )
    }
}

impl FromSql<SmallInt, Pg> for UserRole
where
    i16: FromSql<SmallInt, Pg>,
{
    fn from_sql(bytes: diesel::backend::RawValue<'_, Pg>) -> diesel::deserialize::Result<Self> {
        match <i16 as FromSql<SmallInt, Pg>>::from_sql(bytes)? {
            1 => Ok(Self::SecurityGuard),
            2 => Ok(Self::SecurityHead),
            3 => Ok(Self::SystemAdmin),
            _ => Err("Unrecognized UserRole variant".into()),
        }
    }
}

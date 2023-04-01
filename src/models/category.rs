#[derive(Debug)]
pub enum Category {
    Student = 1,
    Visitor = 2,
    Faculty = 3,
    Staff = 4,
}

use diesel::deserialize::FromSql;
use diesel::pg::Pg;
use diesel::serialize::ToSql;
use diesel::sql_types::SmallInt;
use serde::{Deserialize, Serialize};

impl Serialize for Category {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(match *self {
            Self::Student => 1,
            Self::Visitor => 2,
            Self::Faculty => 3,
            Self::Staff => 4,
        })
    }
}

impl<'de> Deserialize<'de> for Category {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u32::deserialize(deserializer)?;
        match value {
            1 => Ok(Self::Student),
            2 => Ok(Self::Visitor),
            3 => Ok(Self::Faculty),
            4 => Ok(Self::Staff),
            _ => Err(serde::de::Error::custom(format!(
                "invalid value for User role: {value}"
            ))),
        }
    }
}

impl ToSql<SmallInt, Pg> for Category
where
    i16: ToSql<SmallInt, Pg>,
{
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        <i16 as ToSql<SmallInt, Pg>>::to_sql(
            match self {
                Self::Student => &1,
                Self::Visitor => &2,
                Self::Faculty => &3,
                Self::Staff => &4,
            },
            out,
        )
    }
}

impl FromSql<SmallInt, Pg> for Category
where
    i16: FromSql<SmallInt, Pg>,
{
    fn from_sql(bytes: diesel::backend::RawValue<'_, Pg>) -> diesel::deserialize::Result<Self> {
        match <i16 as FromSql<SmallInt, Pg>>::from_sql(bytes)? {
            1 => Ok(Self::Student),
            2 => Ok(Self::Visitor),
            3 => Ok(Self::Faculty),
            4 => Ok(Self::Staff),
            _ => Err("Unrecognized UserRole variant".into()),
        }
    }
}

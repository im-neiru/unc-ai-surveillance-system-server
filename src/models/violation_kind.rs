use diesel::deserialize::FromSql;
use diesel::pg::Pg;
use diesel::serialize::ToSql;
use diesel::sql_types::SmallInt;
use diesel::{AsExpression, FromSqlRow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, AsExpression, FromSqlRow)]
#[diesel(sql_type = SmallInt)]
pub enum ViolationKind {
    FacemaskProtocol = 1,
    FootTraffic = 2,
}

impl Serialize for ViolationKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(match *self {
            ViolationKind::FacemaskProtocol => 1,
            ViolationKind::FootTraffic => 2,
        })
    }
}

impl<'de> Deserialize<'de> for ViolationKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u32::deserialize(deserializer)?;
        match value {
            0 => Ok(Self::FacemaskProtocol),
            1 => Ok(Self::FootTraffic),
            _ => Err(serde::de::Error::custom(format!(
                "invalid value for violation kind: {value}"
            ))),
        }
    }
}

const NUMERIC_VALUES: [i16; 2] = [1, 2];

impl ToSql<SmallInt, Pg> for ViolationKind
where
    i16: ToSql<SmallInt, Pg>,
{
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        <i16 as ToSql<SmallInt, Pg>>::to_sql(
            match self {
                ViolationKind::FacemaskProtocol => &NUMERIC_VALUES[0],
                ViolationKind::FootTraffic => &NUMERIC_VALUES[1],
            },
            out,
        )
    }
}

impl FromSql<SmallInt, Pg> for ViolationKind
where
    i16: FromSql<SmallInt, Pg>,
{
    fn from_sql(bytes: diesel::backend::RawValue<'_, Pg>) -> diesel::deserialize::Result<Self> {
        match <i16 as FromSql<SmallInt, Pg>>::from_sql(bytes)? {
            1 => Ok(Self::FacemaskProtocol),
            2 => Ok(Self::FootTraffic),
            _ => Err("ViolationKind UserRole variant".into()),
        }
    }
}

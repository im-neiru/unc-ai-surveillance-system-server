use diesel::{AsExpression, FromSqlRow};
use diesel::pg::Pg;
use diesel::serialize::ToSql;
use diesel::deserialize::FromSql;
use diesel::sql_types::SmallInt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(AsExpression, FromSqlRow)]
#[diesel(sql_type = SmallInt)]
pub enum UserRole {
    SecurityGuard = 1,
    SecurityHead = 2,
    SystemAdmin = 3,
}

const NUMERIC_VALUES : [i16; 3] = [1, 2, 3];

impl ToSql<SmallInt, Pg> for UserRole where i16: ToSql<SmallInt, Pg> {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
        <i16 as ToSql::<SmallInt, Pg>>::to_sql(match self {
            UserRole::SecurityGuard => &NUMERIC_VALUES[0],
            UserRole::SecurityHead => &NUMERIC_VALUES[1],
            UserRole::SystemAdmin => &NUMERIC_VALUES[2],
        }, out)
    }
}

impl FromSql<SmallInt, Pg> for UserRole where i16: FromSql<SmallInt, Pg> {
    fn from_sql(bytes: diesel::backend::RawValue<'_, Pg>) -> diesel::deserialize::Result<Self> {

        match <i16 as FromSql::<SmallInt, Pg>>::from_sql(bytes)? {
                1 => Ok(Self::SecurityGuard),
                2 => Ok(Self::SecurityHead),
                3 => Ok(Self::SystemAdmin),
                _=> Err("Unrecognized UserRole variant".into())
        }
    }
}
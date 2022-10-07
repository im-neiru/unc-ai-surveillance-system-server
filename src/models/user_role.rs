use diesel::pg::Pg;
use diesel::expression::AsExpression;
use diesel::serialize::ToSql;
use diesel::sql_types::SmallInt;

#[derive(Debug, Clone, Copy)]
#[derive(AsExpression)]
#[diesel(sql_type = SmallInt)]
pub enum UserRole {
    SecurityGuard,
    SecurityHead,
    SystemAdmin,
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
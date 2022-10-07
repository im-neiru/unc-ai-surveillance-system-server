use diesel::backend::Backend;
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

const NUMERIC_VALUES : [u16; 3] = [1, 2, 3];

impl<DB> ToSql<SmallInt, DB> for UserRole where DB: Backend, u16: ToSql<SmallInt, DB> {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, DB>) -> diesel::serialize::Result {
        (match self {
            UserRole::SecurityGuard => &NUMERIC_VALUES[0],
            UserRole::SecurityHead => &NUMERIC_VALUES[1],
            UserRole::SystemAdmin => &NUMERIC_VALUES[2],
        }).to_sql(out)
    }
}
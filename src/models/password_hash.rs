use diesel::AsExpression;
use diesel::backend::Backend;
use diesel::serialize::ToSql;
use diesel::sql_types::Binary;

#[derive(Debug, Clone, Copy)]
#[derive(AsExpression)]
#[diesel(sql_type = Binary)]
pub struct PasswordHash([u8; 64]);

impl From<[u8; 64]> for PasswordHash {
    #[inline]
    fn from(arr: [u8; 64]) -> Self {
        Self(arr)
    }
}

impl<DB> ToSql<Binary, DB> for PasswordHash where DB: Backend, [u8; 64]: ToSql<Binary, DB> {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, DB>) -> diesel::serialize::Result {
        ToSql::<Binary, DB>::to_sql(&self.0, out)
    }
}
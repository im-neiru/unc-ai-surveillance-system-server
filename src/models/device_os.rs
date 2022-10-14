use serde::{Serialize, Deserialize};

use diesel::pg::Pg;
use diesel::AsExpression;
use diesel::serialize::ToSql;
use diesel:: sql_types::SmallInt;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
#[derive(AsExpression)]
#[diesel(sql_type = SmallInt)]
pub enum DeviceOs {
    Android = 1,
    Windows = 2,
    Linux = 3,
}

const NUMERIC_VALUES : [i16; 3] = [1, 2, 3];

impl ToSql<SmallInt, Pg> for DeviceOs where i16: ToSql<SmallInt, Pg> {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
        <i16 as ToSql::<SmallInt, Pg>>::to_sql(match self {
            DeviceOs::Android => &NUMERIC_VALUES[0],
            DeviceOs::Windows => &NUMERIC_VALUES[1],
            DeviceOs::Linux => &NUMERIC_VALUES[2],
        }, out)
    }
}
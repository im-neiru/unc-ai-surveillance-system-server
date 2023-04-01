use diesel::{
    deserialize::FromSqlRow,
    sql_types::{SmallInt, Text},
    Insertable,
};
use serde::Serialize;

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::violations)]
pub struct ViolationUnknownInsert {
    pub area_code: String,
    pub violation_kind: super::ViolationKind,
    pub date_time: chrono::NaiveDateTime,
    pub image_bytes: Vec<u8>,
    pub identified: bool,
}

#[derive(Debug, Serialize)]
pub struct ViolationUnknown {
    pub id: uuid::Uuid,
    #[serde(rename = "area-code")]
    pub area_code: String,
    #[serde(rename = "violation-kind")]
    pub violation_kind: super::ViolationKind,
    #[serde(rename = "date-time")]
    pub date_time: chrono::NaiveDateTime,
}

impl
    FromSqlRow<
        (
            diesel::sql_types::Uuid,
            Text,
            SmallInt,
            diesel::sql_types::Timestamp,
        ),
        diesel::pg::Pg,
    > for ViolationUnknown
{
    fn build_from_row<'a>(
        row: &impl diesel::row::Row<'a, diesel::pg::Pg>,
    ) -> diesel::deserialize::Result<Self> {
        Ok(Self {
            id: row.get_value("id")?,
            area_code: row.get_value("area_code")?,
            violation_kind: row.get_value("violation_kind")?,
            date_time: row.get_value::<diesel::sql_types::Timestamp, chrono::NaiveDateTime, &str>(
                "date_time",
            )?,
        })
    }
}

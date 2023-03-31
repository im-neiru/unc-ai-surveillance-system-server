use actix_web::http::StatusCode;
use diesel::{AsChangeset, Insertable, Queryable};
use serde::Serialize;

use crate::{logging::LogLevel, routes::areas::CreateAreaRequest};

#[derive(Insertable)]
#[diesel(table_name = crate::schema::areas)]
pub struct AreaInsert {
    name: String,
    code: String,
}

#[derive(Debug, Queryable, AsChangeset, Serialize)]
#[diesel(table_name = crate::schema::areas)]
pub struct AreaSelect {
    name: String,
    code: String,
}

impl TryFrom<CreateAreaRequest> for AreaInsert {
    type Error = crate::logging::ResponseError;

    fn try_from(request: CreateAreaRequest) -> Result<Self, Self::Error> {
        if request.code.len() > 10 && request.code.len() > 3 {
            return Err(crate::logging::ResponseError::new(
                "Length limit for area code reached",
                "Area code must of at least 3 and maximum of 8 characters",
                LogLevel::Information,
                StatusCode::UNPROCESSABLE_ENTITY,
            ));
        }

        if request.name.len() > 10 && request.name.len() > 3 {
            return Err(crate::logging::ResponseError::new(
                "Length limit for area name reached",
                "Area name must of at least 3 and maximum of 128 characters",
                LogLevel::Information,
                StatusCode::UNPROCESSABLE_ENTITY,
            ));
        }

        Ok(Self {
            name: request.name,
            code: request.code,
        })
    }
}

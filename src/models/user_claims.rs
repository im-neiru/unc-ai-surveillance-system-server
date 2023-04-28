use std::future::Future;
use std::pin::Pin;

use actix_web::dev::Payload;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::HttpRequest;

use diesel::{AsChangeset, ExpressionMethods, QueryDsl, RunQueryDsl};

use crate::data::AppData;
use crate::logging::{LogLevel, ResponseError};
use crate::models::UserRole;

#[derive(Debug)]
pub struct UserClaims {
    pub session_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub assigned_role: super::UserRole,
}

impl actix_web::FromRequest for UserClaims {
    type Error = ResponseError;

    type Future = Pin<Box<dyn Future<Output = Result<Self, ResponseError>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let req = req.clone();

        Box::pin(async move {
            let token = req
                .cookie("jwt")
                .ok_or(ResponseError::new(
                    "JSON Web token not found",
                    "Invalid Session",
                    LogLevel::Information,
                    StatusCode::UNAUTHORIZED,
                ))?
                .value()
                .to_owned();

            let state = req.app_data::<Data<AppData>>().unwrap();
            let jwtc = state.jwt_decode(&token)?;

            use crate::schema::sessions;
            use crate::schema::users;

            let mut database = state.connect_database();

            let (user_id, assigned_role): (uuid::Uuid, UserRole) = users::table
                .inner_join(sessions::table)
                .filter(users::deactivated.eq(false))
                .filter(sessions::id.eq(jwtc.session_id))
                .select((users::id, users::assigned_role))
                .get_result(&mut database)
                .or(Err(ResponseError::new(
                    "User do not found",
                    "Invalid Session",
                    LogLevel::Information,
                    StatusCode::UNAUTHORIZED,
                )))?;

            Ok(Self {
                session_id: jwtc.session_id,
                user_id,
                assigned_role,
            })
        })
    }
}

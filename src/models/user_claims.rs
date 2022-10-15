use std::pin::Pin;
use std::future::Future;

use actix_web::web::Data;
use actix_web::{HttpRequest};
use actix_web::dev::Payload;

use diesel::{Queryable, AsChangeset, QueryDsl, RunQueryDsl, ExpressionMethods};

use crate::data::AppData;

#[derive(Debug, Queryable, AsChangeset)]
#[diesel(table_name = crate::schema::users)]
pub struct UserClaims {
    pub id: uuid::Uuid,
    pub assigned_role: super::UserRole,
}

impl actix_web::FromRequest for UserClaims {
    type Error = actix_web::Error;

    type Future = Pin<Box<dyn Future<Output = Result<Self, actix_web::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let req = req.clone();
        println!("Test");
        Box::pin(async move {
            let token = req.cookie("jwt")
                .unwrap()
                .value()
                .to_owned();
            
            let state = req.app_data::<Data<AppData>>().unwrap();
            let jwtc = state.jwt_decode(&token);

            use crate::schema::sessions;
            use crate::schema::users;

            let database = &mut *state.connect_database();

            let user_claims: Self = users::table.inner_join(sessions::table)
                .filter(sessions::id.eq(jwtc.session_id))
                .select((users::id, users::assigned_role))
                .get_result(database).unwrap();
            
            Ok(user_claims)
        })
    }

}
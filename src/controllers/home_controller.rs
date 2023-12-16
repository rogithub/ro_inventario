
use askama_actix::{ TemplateToResponse };
use actix_web::{ get, Responder };
use crate::models::home_models::{ LandingModel };
use actix_identity::{Identity};
use actix_web::web;
use crate::app_state::AppState;


#[get("/landing")]
pub async fn landing(identity: Option<Identity>, data: web::Data<AppState>) -> impl Responder {
    AppState::identity_check(identity).unwrap();

    let model = LandingModel::default();
    model.to_response()
/*
    match AppState::identity_check(identity) {
        Ok(claims) => {
            // use claims for authorization or other purposes
            let model = LandingModel::default();
            Ok(model.to_response())
        },
        Err(IdentityError::MissingId) => {
            log::warn!("Missing user identity");
            Err(actix_web::error::Unauthorized()) // or custom unauthorized response
        },
        Err(IdentityError::InvalidId(err)) => {
            log::error!("Invalid user identity: {}", err);
            Err(actix_web::error::InternalServerError::from(err)) // or custom error response
        },
    }
*/
}

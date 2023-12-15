
use askama_actix::{ TemplateToResponse };
use actix_web::{ get,  error, Responder };
use crate::models::home_models::{ LandingModel };

use actix_identity::{Identity};


#[get("/landing")]
pub async fn landing(identity: Option<Identity>) -> impl Responder {
    let id = match identity.map(|id| id.id()) {
        None => "anonymous".to_owned(),
        Some(Ok(id)) => id,
        Some(Err(err)) => return Err(error::ErrorInternalServerError(err)),
    };

    let model = LandingModel::default();    
    Ok(model.to_response())

    //Ok(format!("Hello {id}"))
}

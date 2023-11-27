use askama_actix::{ TemplateToResponse };
use actix_web::{ web, get, post,  error, Responder, HttpRequest };
use log::{info};
use crate::models::account_models::{ LoginModel, Validator };
use actix_web::{
    HttpMessage as _,
    http::StatusCode
};


use actix_identity::{Identity};


#[get("/index")]
pub async fn index(identity: Option<Identity>) -> impl Responder {
    let id = match identity.map(|id| id.id()) {
        None => "anonymous".to_owned(),
        Some(Ok(id)) => id,
        Some(Err(err)) => return Err(error::ErrorInternalServerError(err)),
    };

    Ok(format!("Hello {id}"))
}

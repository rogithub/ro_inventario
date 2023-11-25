use askama_actix::{ TemplateToResponse };
use actix_web::{ web, get, post, Responder };
use log::{info};
use crate::models::account_models::{ LoginModel, Validator };


#[get("/login")]
pub async fn login() -> impl Responder {
    let model = LoginModel::default();    
    model.to_response()
}

#[post("/submit")]
pub async fn submit(mut form: web::Form<LoginModel>) -> impl Responder {    
    form.validate();  
    info!("{:?}", form);
    form.to_response()
}
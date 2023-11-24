use askama_actix::{ TemplateToResponse };
use actix_web::{ web, get, post, Responder };
use log::{info};
use crate::models::account_models::{ LoginModel };


#[get("/login")]
pub async fn login() -> impl Responder {
    let model = LoginModel { email: "".to_string(), password: "".to_string() };    
    model.to_response()
}

#[post("/submit")]
pub async fn submit(form: web::Form<LoginModel>) -> impl Responder {
    info!("{:?}", form);
    form.to_response()
}
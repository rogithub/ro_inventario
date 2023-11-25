use askama_actix::{ TemplateToResponse };
use actix_web::{ web, get, post, Responder };
use log::{info};
use crate::models::account_models::{ LoginModel, Validate };


#[get("/login")]
pub async fn login() -> impl Responder {
    let model = LoginModel::default();    
    model.to_response()
}

#[post("/submit")]
pub async fn submit(form: web::Form<LoginModel>) -> impl Responder { 
    let model = LoginModel { 
        email: form.email.clone(), 
        password : form.password.clone()
    };
    let errors = model.run();   
    info!("{:?}", form);
    info!("{:?}", errors);
    form.to_response()
}
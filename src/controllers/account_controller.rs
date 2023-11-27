use askama_actix::{ TemplateToResponse };
use actix_web::{ web, get, post, Responder, HttpRequest };
use log::{info};
use crate::models::account_models::{ LoginModel, Validator };
use actix_web::{
    HttpMessage as _,
    http::StatusCode
};


use actix_identity::{Identity};


#[get("/login")]
pub async fn login() -> impl Responder {
    let model = LoginModel::default();    
    model.to_response()
}

#[post("/submit")]
pub async fn submit(mut form: web::Form<LoginModel>, req: HttpRequest) -> impl Responder {    
    form.validate();  
    info!("{:?}", form);

    //if validation error
    //form.to_response()

    Identity::login(&req.extensions(), "user1".to_owned()).unwrap();

    web::Redirect::to("/home/index").using_status_code(StatusCode::FOUND)
}


#[post("/logout")]
async fn logout(id: Identity) -> impl Responder {
    id.logout();

    web::Redirect::to("/").using_status_code(StatusCode::FOUND)
}
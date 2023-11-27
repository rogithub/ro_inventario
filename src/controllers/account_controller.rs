use askama_actix::{ TemplateToResponse };
use actix_web::{ web, get, post, Responder, HttpRequest,HttpMessage as _,
    http::StatusCode };
use log::{info};
use actix_web::web::Redirect;
use crate::models::account_models::{ LoginModel, Validator };


use actix_identity::{Identity};


#[get("/login")]
pub async fn login() -> impl Responder {
    let model = LoginModel::default();    
    model.to_response()
}

#[post("/submit")]
pub async fn submit(mut form: web::Form<LoginModel>, req: HttpRequest) -> impl Responder {    
    let is_valid = form.validate();  
    info!("{:?}", form);

    if is_valid {
        Identity::login(&req.extensions(), "user1".to_owned()).unwrap();
        return Redirect::to("/home/index").using_status_code(StatusCode::FOUND)    
    }
    
    //if validation esrror
    form.to_response()    
}


#[post("/logout")]
async fn logout(id: Identity) -> impl Responder {
    id.logout();

    Redirect::to("/").using_status_code(StatusCode::FOUND)
}
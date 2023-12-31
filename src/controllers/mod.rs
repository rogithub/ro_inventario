use actix_web::{ get, Responder };
use askama_actix::{ Template, TemplateToResponse };
pub mod account_controller;
pub mod home_controller;

#[derive(Template)]
#[template(path = "public.html")]
struct PublicTemplate;

#[get("/")]
pub async fn index() -> impl Responder {
    let model = PublicTemplate;
    model.to_response()
}

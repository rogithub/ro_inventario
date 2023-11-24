use actix_web::{ get, Responder };
use askama_actix::{ Template, TemplateToResponse };
pub mod account;

#[derive(Template)]
#[template(path = "public.html")]
struct PublicTemplate<'a> {
    name: &'a str,
}

#[get("/")]
pub async fn index() -> impl Responder {
    let model = PublicTemplate { name: "Papelería" };
    model.to_response()
}

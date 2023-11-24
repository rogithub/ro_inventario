use askama_actix::{ Template, TemplateToResponse };
use actix_web::{ get, Responder };

#[derive(Template)]
#[template(path = "login.html")]
struct PublicTemplate;


#[get("/login")]
pub async fn login() -> impl Responder {    
    let model = PublicTemplate;
    model.to_response()
}

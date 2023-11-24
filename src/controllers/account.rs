use askama_actix::{ Template, TemplateToResponse };
use actix_web::{ get, Responder };

#[derive(Template)]
#[template(path = "public.html")]
struct PublicTemplate<'a> {
    name: &'a str,
}


#[get("/login")]
pub async fn login() -> impl Responder {    
    let model = PublicTemplate { name: "Login" };
    model.to_response()
}

use askama::Template;
use actix_web::{ HttpResponse, Responder };

#[derive(Template)]
#[template(path = "public.html")]
struct PublicTemplate<'a> {
    name: &'a str,
}

pub async fn login() -> impl Responder {
    let model = PublicTemplate { name: "Login" };    
    HttpResponse::Ok()
    .body(model.render().unwrap())
}

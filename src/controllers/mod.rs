use askama::Template;
use actix_web::{ get, HttpResponse, Responder };

#[derive(Template)]
#[template(path = "public.html")]
struct PublicTemplate<'a> {
    name: &'a str,
}

#[get("/")]
pub async fn index() -> impl Responder {
    let model = PublicTemplate { name: "Papelería" };    
    HttpResponse::Ok()
    .body(model.render().unwrap())
}

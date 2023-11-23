use askama::Template;
use actix_web::{ get, HttpResponse, Responder };

#[derive(Template)]
#[template(path = "_layout.html")]
struct LayoutTemplate<'a> {
    name: &'a str,
}

#[get("/")]
pub async fn index() -> impl Responder {
    let hello = LayoutTemplate { name: "Contenido" };    
    HttpResponse::Ok()
    .body(hello.render().unwrap())
}

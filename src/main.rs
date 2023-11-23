use actix_web::{
    get, App, HttpResponse, HttpServer, Responder,
};
use askama::Template;
mod settings;



#[derive(Template)]
#[template(path = "_layout.html")]
struct LayoutTemplate<'a> {
    name: &'a str,
}

#[get("/")]
async fn index() -> impl Responder {
    let hello = LayoutTemplate { name: "Contenido" };    
    HttpResponse::Ok()
    .body(hello.render().unwrap())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    let settings = settings::load().expect("could not load settins file");
    println!("server is now running");
    println!("http://{}:{}", settings.hosting.ip, settings.hosting.port);

    HttpServer::new(|| {
        App::new()
            .service(index)
    })
    .bind((settings.hosting.ip, settings.hosting.port))?
    .run()
    .await
}
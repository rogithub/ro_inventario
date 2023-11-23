use actix_web::{
    get, App, HttpResponse, HttpServer, Responder,
};
use askama::Template;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::env;
use std::path::{Path};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Settings {
   environment: String,
   hosting: Hosting  
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Hosting {
   ip: String,
   port: u16
}

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
    let current_dir = env::current_dir()?;
    let settings_file = Path::new("settings/dev.json");

    let settings_path = current_dir.join(settings_file);
    
    let file = File::open(settings_path).expect("Unable to open settings file");
    let settings: Settings = serde_json::from_reader(file).expect("settings JSON was not well-formatted");

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
use actix_web::{
    App, HttpServer
};
use actix_web::middleware::Logger;
use env_logger::Env;
use log::{info};
use actix_files::Files;

mod settings;
mod controllers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let settings = settings::load().expect("Could not load settings file");
    info!("server running at");
    info!("http://{}:{}", settings.hosting.ip, settings.hosting.port);

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .service(controllers::index)
            .service(Files::new("/static", "./wwwroot").prefer_utf8(true))
            //.service(Files::new("/static", "./wwwroot").prefer_utf8(true).show_files_listing())
    })
    .bind((settings.hosting.ip, settings.hosting.port))?
    .run()
    .await
}
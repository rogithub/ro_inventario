use actix_web::{
    App, HttpServer, web
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

        let account = web::scope("/account").service(controllers::account::login);

        App::new()            
            // routes starting by
            .service(controllers::index)            
            .service(account)

            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))            
            .service(Files::new("/", "./wwwroot").prefer_utf8(true))
            
    })
    .bind((settings.hosting.ip, settings.hosting.port))?
    .run()
    .await
}
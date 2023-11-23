use actix_web::{
    App, HttpServer
};
use actix_web::middleware::Logger;
use env_logger::Env;

mod settings;
mod controllers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let settings = settings::load().expect("Could not load settings file");
    println!("server running at");
    println!("http://{}:{}", settings.hosting.ip, settings.hosting.port);

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .service(controllers::index)
    })
    .bind((settings.hosting.ip, settings.hosting.port))?
    .run()
    .await
}
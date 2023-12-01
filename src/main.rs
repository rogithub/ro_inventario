
use actix_web::middleware::Logger;
use env_logger::Env;
use log::{info};
use actix_files::Files;

mod settings;
mod controllers;
mod models;
mod validators;
mod app_state;
mod repos;

use actix_identity::IdentityMiddleware;
use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::{time::Duration, Key},   
    middleware, web, App, HttpServer
};

const ONE_MINUTE: Duration = Duration::minutes(1);

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    env_logger::init_from_env(Env::default().default_filter_or("info"));
    
    // Generate a random secret key. Note that it is important to use a unique
    // secret key for every project. Anyone with access to the key can generate
    // authentication cookies for any user!
    //
    // If the secret key is read from a file or the environment, make sure it is generated securely.
    // For example, a secure random key (in base64 format) can be generated with the OpenSSL CLI:
    // ```
    // openssl rand -base64 64
    // ```
    //
    // Then decoded and used converted to a Key:
    // ```
    // let secret_key = Key::from(base64::decode(&private_key_base64).unwrap());
    // ```
    let secret_key = Key::generate();


    let settings = settings::load().expect("Could not load settings file");
    let s = settings::load().expect("Could not load settings file");    
    let app_state = app_state::AppState { settings: settings };    
    info!("server running at");
    info!("{:?}", s.hosting);

    HttpServer::new(move || {

        let account = web::scope("/account")
                        .service(controllers::account_controller::login)
                        .service(controllers::account_controller::submit)
                        .service(controllers::account_controller::logout);

        let home = web::scope("/home")
                        .service(controllers::home_controller::index);

        App::new()
            .app_data(web::Data::new(app_state.clone()))

            // routes starting by
            .service(controllers::index)            
            .service(account)
            .service(home)
            .service(Files::new("/", "./wwwroot").prefer_utf8(true))

            // cookie auth & identity
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_name("auth-inventario".to_owned())
                    .cookie_secure(false)
                    .session_lifecycle(PersistentSession::default().session_ttl(ONE_MINUTE))
                    .build(),
            )


            // logger
            .wrap(middleware::NormalizePath::trim())
            .wrap(middleware::Logger::default())            
            .wrap(Logger::new("%a %{User-Agent}i"))            
            
    })
    // In general, use 127.0.0.1:<port> when testing locally and 0.0.0.0:<port> when deploying 
    // (with or without a reverse proxy or load balancer) so that the server is accessible.
    .bind((s.hosting.ip, s.hosting.port))?
    .run()
    .await
}
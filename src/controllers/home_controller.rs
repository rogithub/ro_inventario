
use actix_web::{ get,  error, Responder };


use actix_identity::{Identity};


#[get("/index")]
pub async fn index(identity: Option<Identity>) -> impl Responder {
    let id = match identity.map(|id| id.id()) {
        None => "anonymous".to_owned(),
        Some(Ok(id)) => id,
        Some(Err(err)) => return Err(error::ErrorInternalServerError(err)),
    };

    Ok(format!("Hello {id}"))
}

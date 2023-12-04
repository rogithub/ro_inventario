use askama_actix::{ TemplateToResponse };
use actix_web::{ 
    web, get, post, Either,
    Responder, HttpRequest, HttpMessage as _,
    http::StatusCode };
use log::{info};
use actix_web::web::Redirect;
use crate::models::account_models::{ LoginModel, Validator };
use crate::app_state::AppState;
use crate::repos::users_repo::{UserEntity};


use actix_identity::{Identity};


#[get("/login")]
pub async fn login() -> impl Responder {
    let model = LoginModel::default();    
    model.to_response()
}

#[post("/submit")]
pub async fn submit(mut form: web::Form<LoginModel>, req: HttpRequest, data: web::Data<AppState>) -> Either<impl Responder, Redirect> {    
    let is_valid = form.validate();  
    info!("{:?}", form);

    let db_pool = data.connect().await;
    let maybe_entity = UserEntity::get_one(db_pool, form.email.clone().as_str()).await;    
    
    let db_pool = data.connect().await;
    if is_valid && UserEntity::has_access(db_pool, &maybe_entity, &form.password).await {
        let entity = maybe_entity.unwrap();
        info!("ENTITY {:?}", entity);
        Identity::login(&req.extensions(), form.email.clone()).unwrap();
        return Either::Right(Redirect::to("/home/index").using_status_code(StatusCode::FOUND))
    }
    form.server_err = Some("Su contraseña es incorrecta".to_string());
    //if validation error
    Either::Left(form.to_response())    
}


#[post("/logout")]
async fn logout(id: Identity) -> impl Responder {
    id.logout();

    Redirect::to("/").using_status_code(StatusCode::FOUND)
}
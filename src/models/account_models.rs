use askama_actix::{ Template };
use serde::Deserialize;


#[derive(Template, Deserialize, Debug)]
#[template(path = "login.html")]
pub struct LoginModel {
    pub email:  String,
    pub password:  String
}
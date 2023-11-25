use askama_actix::{ Template };
use serde::Deserialize;
use crate::validators::str_validators::is_empty_string;

#[derive(Template, Deserialize, Debug, Default)]
#[template(path = "login.html")]
pub struct LoginModel {
    pub email:  String,
    pub password:  String,
    pub email_err: Option<String>,
    pub password_err: Option<String> 
}

pub trait Validator {
    fn validate(&mut self) -> ();
}
 
impl Validator for LoginModel {
    fn validate(&mut self) -> () {
        if is_empty_string(&self.email) {
            self.email_err = Some(String::from("El correo electrónico no puede estar vacío"))        
        } else {
            self.email_err = None
        }

        if is_empty_string(&self.password) {
            self.password_err = Some(String::from("La contraseña no puede estar vacía"))        
        } else {
            self.password_err = None
        }
    }    
}

use askama_actix::{ Template };
use serde::Deserialize;
use std::collections::HashMap;
use regex::Regex;

#[derive(Template, Deserialize, Debug, Default)]
#[template(path = "login.html")]
pub struct LoginModel {
    pub email:  String,
    pub password:  String
}

pub trait Validate {
    fn run(&self) -> HashMap<String, String>;    
}

fn validate_email(email_address: &str) -> bool {
    let email_regex = Regex::new(r"^\w+([-+.']\w+)*@\w+([-.]\w+)*\.\w+([-.]\w+)*$").unwrap();
    let is_valid = email_regex.is_match(email_address);
    is_valid
}

/// We only validate min length 8 max length 255
fn validate_password(pwd: &str) -> bool {
    let pwd_regex = Regex::new(r"^.{8,255}$").unwrap();
    let is_valid = pwd_regex.is_match(pwd);
    is_valid
}

fn is_empty_string(val: &str) -> bool {
    let empty_str_regex = Regex::new(r"^\s*$").unwrap();
    let is_valid = empty_str_regex.is_match(val);
    is_valid
}

 
impl Validate for LoginModel {
    fn run(&self) -> HashMap<String, String> {
        let mut errors: HashMap<String, String> = HashMap::new();
        if is_empty_string(&self.email) {
            errors.insert
            (
                "email".to_string(),
                "El correo electrónico no puede estar vacío".to_string()
            );
        }
        if is_empty_string(&self.password) {
            errors.insert
            (
                "password".to_string(),
                "El password no puede estar vacío".to_string()
            );
        }

        errors
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string_is_invalid_email() {
        let is_valid = validate_email("");
        assert_eq!(is_valid, false);
    }

    #[test]
    fn email_can_contain_dots() {
        let is_valid = validate_email("s.o.s@sos.org");
        assert_eq!(is_valid, true);
    }

    #[test]
    fn password_complexity_min_length() {
        let is_valid = validate_password("");
        assert_eq!(is_valid, false);

        let is_valid = validate_password("len");
        assert_eq!(is_valid, false);

        let is_valid = validate_password("Tres0!");
        assert_eq!(is_valid, false);

        let is_valid = validate_password("Tres0!78");
        assert_eq!(is_valid, true);
    }

    #[test]
    fn not_empty() {
        let it_is = is_empty_string("");
        assert_eq!(it_is, true);

        let it_is = is_empty_string("  ");
        assert_eq!(it_is, true);

        let it_is = is_empty_string("   ");
        assert_eq!(it_is, true);

        let it_is = is_empty_string("
        ");
        assert_eq!(it_is, true);

        let it_is = is_empty_string("no");
        assert_eq!(it_is, false);

        let it_is = is_empty_string(" casi pero ... no ");
        assert_eq!(it_is, false);

        let it_is = is_empty_string(".");
        assert_eq!(it_is, false);
    }
}
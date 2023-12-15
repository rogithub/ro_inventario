use askama_actix::{ Template };
use serde::Deserialize;

#[derive(Template, Deserialize, Debug, Default)]
#[template(path = "landing.html")]
pub struct LandingModel {    
}

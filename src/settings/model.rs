use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Settings {
   pub db: String,
   pub hosting: Hosting  
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Hosting {
   pub protocol: String,
   pub ip: String,
   pub port: u16
}

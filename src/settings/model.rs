use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Settings {
   pub db: String,
   pub hosting: Hosting  
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Hosting {
   pub protocol: String,
   pub ip: String,
   pub port: u16
}

impl fmt::Debug for Hosting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}://{}:{}", self.protocol, self.ip, self.port)
    }
}


impl Settings {
    pub fn db_url(&self) -> String
    {
        format!("sqlite://./{0}", &self.db.clone())
    }
}
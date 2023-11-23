use serde::{Deserialize, Serialize};
use std::fs::File;
use std::env;
use std::path::{Path};
use std::error::Error;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Settings {
   pub environment: String,
   pub hosting: Hosting  
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Hosting {
   pub ip: String,
   pub port: u16
}


pub fn load() -> Result<Settings, Box<dyn Error>>
{
   let current_dir = env::current_dir()?;
   let settings_file = Path::new("settings/dev.json");

   let settings_path = current_dir.join(settings_file);
   
   let file = File::open(settings_path).expect("Unable to open settings file");
   let settings: Settings = serde_json::from_reader(file).expect("settings JSON was not well-formatted");   

   Ok(settings)
}
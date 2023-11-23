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
   let settings_dir = env::current_dir()?.join(Path::new("settings"));   
   
   let key = "INVENTARIO_ENV";
   let default_env = "development";
   let file_name: String;
   let settings_file = match env::var(key) {
      Ok(val) => {
         file_name = format!("{}.json", val);
         Path::new(&file_name)
      },
      Err(_) => {         
         file_name = format!("{}.json", default_env);
         env::set_var(key, default_env);
         Path::new(&file_name)
      },
   };

   let settings_path = settings_dir.join(settings_file);   
   
   let file = File::open(settings_path).expect("Unable to open settings file");
   let settings: Settings = serde_json::from_reader(file).expect("settings JSON was not well-formatted");   

   Ok(settings)
}
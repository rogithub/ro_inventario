use std::fs::File;
use std::env;
use std::path::{Path};
use std::error::Error;

pub mod model;


pub fn load() -> Result<model::Settings, Box<dyn Error>>
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
   
   let file = File::open(settings_path.clone()).expect(&format!("Unable to open settings file {:?}", settings_path));
   let settings: model::Settings = serde_json::from_reader(file).expect(&format!("Settings JSON was not well-formatted {:?}", settings_path));

   Ok(settings)
}
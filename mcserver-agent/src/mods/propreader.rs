use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter, Seek, SeekFrom};

pub struct ServerProperties {
    properties: HashMap<String, String>,
    pub dir: String,
}

impl ServerProperties {
    pub fn new(dir: &str) -> Result<Self, Box<dyn Error>> {
        let reader = BufReader::new(File::open(format!("{}/server.properties", dir))?);
        let properties = java_properties::read(reader)?;
        Ok(ServerProperties {
            properties,
            dir: dir.to_string(),
        })
    }
    pub fn get(&self, key: &str) -> Option<&String> {
        self.properties.get(key)
    }
    pub fn set(&mut self, key: &str, value: &str) -> Result<(), Box<dyn Error>> {
        if let None = self.properties.get(key) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Key {} not found in properties", key),
            )));
        }
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .open(format!("{}/server.properties", self.dir))?;

        {
            let mut temp = self.properties.clone();
            temp.insert(key.to_string(), value.to_string());
            let mut writer = BufWriter::new(&file);
            if let Err(e) = java_properties::write(&mut writer, &temp) {
                print!("Failed to write properties: {}", e);
                return Err(Box::new(e));
            }
        }

        file.seek(SeekFrom::Start(0))?;
        let reader = BufReader::new(file);
        self.properties = java_properties::read(reader)?;
        Ok(())
    }
    pub fn update(&mut self) -> Result<(), Box<dyn Error>> {
        let reader = BufReader::new(File::open(format!("{}/server.properties", self.dir))?);
        self.properties = java_properties::read(reader)?;
        Ok(())
    }
}

use crate::mods::propreader::ServerProperties;
use std::path::Path;
use std::process::*;
use std::{io::*, path};

pub fn stop_server(mut stdin: ChildStdin) {
    let _result = writeln!(stdin, "stop");
}

pub struct ServerProcess {
    xms: u32,
    xmx: u32,
    dir: String,
    jar: String,
    properties: Option<ServerProperties>,
    child: Option<Child>,
}

impl Default for ServerProcess {
    fn default() -> Self {
        let props = if Path::new("./server.properties").exists() {
            ServerProperties::new("./").ok()
        } else {
            None
        };
        Self {
            xms: 1024,
            xmx: 1024,
            dir: String::from("./"),
            jar: String::from("server.jar"),
            properties: props,
            child: None,
        }
    }
}
impl ServerProcess {
    pub fn start_server(&mut self) -> Result<()> {
        let child = Command::new("java")
            .current_dir(&self.dir)
            .arg(format!("-Xmx{}M", self.xmx))
            .arg(format!("-Xms{}M", self.xms))
            .arg("-jar")
            .arg(&self.jar)
            .arg("nogui")
            .stdin(Stdio::piped())
            .spawn()?;
        self.child = Some(child);
        self.update_properties();
        Ok(())
    }

    pub fn stop_server(&mut self) -> Result<()> {
        if let Some(child) = &mut self.child {
            if let Some(stdin) = &mut child.stdin {
                writeln!(stdin, "stop")?;
            }
            let _ = child.wait();
        } else {
            println!("No server process to stop");
        }
        self.child = None;
        self.update_properties();
        Ok(())
    }

    pub fn xms(mut self, xms: u32) -> Self {
        self.xms = xms;
        self
    }
    pub fn xmx(mut self, xmx: u32) -> Self {
        self.xmx = xmx;
        self
    }
    pub fn dir(mut self, dir: String) -> Self {
        self.dir = dir;
        self
    }
    pub fn jar(mut self, jar: String) -> Self {
        self.jar = jar;
        self
    }
    pub fn update_properties(&mut self) -> &Self {
        let path_str = format!("{}/server.properties", self.dir);
        let path = Path::new(&path_str);

        if path.exists() {
            match self.properties.as_mut() {
                Some(props) if props.dir == self.dir => {
                    if let Err(e) = props.update() {
                        println!("Failed to update server properties: {}", e);
                    }
                }
                _ => {
                    self.properties = ServerProperties::new(&self.dir).ok();
                }
            }
        } else {
            self.properties = None;
        }

        self
    }
}

use std::io::*;
use std::process::*;

pub fn start_server() -> std::io::Result<Child> {
    Command::new("java")
        .current_dir("/home/oreo/mcserver")
        .arg("-Xmx1024M")
        .arg("-Xms1024M")
        .arg("-jar")
        .arg("server.jar")
        .arg("nogui")
        .stdin(Stdio::piped())
        .spawn()
}

pub fn stop_server(mut stdin: ChildStdin) {
    let _result = writeln!(stdin, "stop");
}

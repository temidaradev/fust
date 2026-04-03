use std::fs::{self, File};
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Linux {
    hostname: String,
    os: String,
    kernel: String,
    cpu: String,
    memory: String,
}

fn get_os_info() -> Option<String> {
    let file = File::open("/etc/os-release").ok()?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        if let Ok(l) = line
            && l.starts_with("PRETTY_NAME=")
        {
            let name = l.replace("PRETTY_NAME=", "").replace("\"", "");
            return Some(name);
        }
    }
    None
}

pub fn show_info() {
    if let Ok(os) = fs::read_to_string("/etc/hostname") {
        println!("\x1b[34mHost:\x1b[0m {}", os.trim());
    }
    if let Some(os) = get_os_info() {
        println!("\x1b[34mOS:\x1b[0m {}", os);
    }
}

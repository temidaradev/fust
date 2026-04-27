use std::env;
use std::io::StdoutLock;
use std::io::BufWriter;
use std::process::Command;
use crate::ascii::{colorize, get_ascii_art, visible_width, Distro};

pub fn get_os_info() -> Option<String> {
    let output = Command::new("sw_vers").arg("-productName").output().ok()?;

    let product_name = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let output = Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .ok()?;

    let product_version = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let output = Command::new("sw_vers").arg("-buildVersion").output().ok()?;

    let build_version = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let output = Command::new("uname").arg("-m").output().ok()?;

    let machine = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Some(format!(
        "{} {} ({}) {}",
        product_name, product_version, build_version, machine
    ))
}

fn get_host_info() -> Option<String> {
    let user = env::var("USER").unwrap_or_else(|_| "user".to_string());
    let output = Command::new("hostname").output().ok()?;
    let host = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let hostname = format!("{}@{}", user, host);

    Some(hostname)
}

fn cpu_info() -> Option<String> {
    let out = std::process::Command::new("sysctl")
        .args(["-n", "machdep.cpu.brand_string"])
        .output()
        .unwrap();

    let raw = String::from_utf8_lossy(&out.stdout).trim().to_string();

    let out = Command::new("sysctl")
        .arg("-n")
        .arg("hw.physicalcpu")
        .output()
        .expect("Failed to execute command");

    let cores = String::from_utf8_lossy(&out.stdout).trim().to_string();

    Some(format!("{} ({})", raw, cores))
}

fn gpu_info() -> Option<String> {
    let output = Command::new("system_profiler")
        .arg("SPDisplaysDataType")
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut model = String::new();
    let mut cores = String::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with("Chipset Model:") {
            model = line.replace("Chipset Model:", "").trim().to_string();
        } else if line.starts_with("Total Number of Cores:") {
            cores = line
                .replace("Total Number of Cores:", "")
                .trim()
                .to_string();
        }
    }

    Some(format!("{} ({})", model, cores))
}

fn uptime_info() -> Option<String> {
    let out = std::process::Command::new("sysctl")
        .args(["-n", "kern.boottime"])
        .output()
        .ok()?;
    
    let raw = String::from_utf8_lossy(&out.stdout);
    let sec_str = raw.split("sec = ").nth(1)?.split(',').next()?;
    let boot_sec: u64 = sec_str.trim().parse().ok()?;
    
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
    let uptime_sec = now.saturating_sub(boot_sec);
    
    Some(format!("{:.2} hours", uptime_sec as f64 / 3600.0))
}

fn disk_info() -> Option<String> {
    let out = std::process::Command::new("df")
        .args(["-k", "/"])
        .output()
        .ok()?;
        
    let raw = String::from_utf8_lossy(&out.stdout);
    let mut lines = raw.lines();
    lines.next()?; // Skip header
    let line = lines.next()?;
    let mut parts = line.split_whitespace();
    
    parts.next()?; // Filesystem
    let total_k: f64 = parts.next()?.parse().ok()?;
    let used_k: f64 = parts.next()?.parse().ok()?;
    
    let gib = |k: f64| k / 1_048_576.0;
    
    Some(format!("{:.2} / {:.2} GiB", gib(used_k), gib(total_k)))
}

fn battery_info() -> Option<String> {
    let out = std::process::Command::new("pmset")
        .args(["-g", "batt"])
        .output()
        .ok()?;
        
    let raw = String::from_utf8_lossy(&out.stdout);
    for line in raw.lines() {
        if line.contains("InternalBattery") {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() > 1 {
                let info = parts[1].split(';').map(|s| s.trim()).collect::<Vec<&str>>();
                if info.len() >= 2 {
                    let percent = info[0];
                    let status = info[1];
                    return Some(format!("{} ({})", percent, status));
                }
            }
        }
    }
    None
}

pub fn show_info(_out: &mut BufWriter<StdoutLock>) {
    let colored_art = colorize(get_ascii_art(&Distro::MacOS));
    let art: Vec<&str> = colored_art.lines().collect();
    let mut infos: Vec<String> = Vec::new();

    let blue = "\x1b[34m";
    let reset = "\x1b[0m";

    if let Some(hostname) = get_host_info() {
        infos.push(format!("{blue}========================={reset}"));
        infos.push(format!("{blue}\u{f108} Host:{reset} {}", hostname));
    }

    if let Some(os) = get_os_info() {
        infos.push(format!("{blue}\u{f17c} OS:{reset} {}", os));
    }

    if let Some(uptime) = uptime_info() {
        infos.push(format!("{blue}\u{f017} Uptime:{reset} {}", uptime));
    }

    if let Some(cpu) = cpu_info() {
        infos.push(format!("{blue}\u{f2db} CPU:{reset} {}", cpu));
    }

    if let Some(gpu) = gpu_info() {
        infos.push(format!("{blue}\u{f26c} GPU:{reset} {}", gpu));
    }

    if let Some(disk) = disk_info() {
        infos.push(format!("{blue}\u{f0a0} Disk:{reset} {}", disk));
    }

    if let Some(battery) = battery_info() {
        infos.push(format!("{blue}\u{f240} Battery:{reset} {}", battery));
    }

    let art_width = art.iter().map(|l| visible_width(l)).max().unwrap_or(0);
    let max_lines = art.len().max(infos.len());

    for i in 0..max_lines {
        let art_line = art.get(i).copied().unwrap_or("");
        let info_line = infos.get(i).map(|s| s.as_str()).unwrap_or("");
        let pad = art_width.saturating_sub(visible_width(art_line));
        println!("{}{}  {}", art_line, " ".repeat(pad), info_line);
    }
}

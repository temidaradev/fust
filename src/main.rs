#[cfg(target_os = "macos")]
mod darwin;
#[cfg(target_os = "linux")]
mod linux;

fn main() {
    #[cfg(target_os = "linux")]
    linux::show_info();

    #[cfg(target_os = "macos")]
    darwin::show_info();
}

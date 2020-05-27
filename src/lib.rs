#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use windows::*;

#[no_mangle]
pub unsafe extern "C" fn husk_hello() {
    #[cfg(target_os = "windows")]
    println!("Hello from husk on Windows!");

    #[cfg(target_os = "macos")]
    println!("Hello from husk on macOS!");

    #[cfg(target_os = "linux")]
    println!("Hello from husk on Linux!");
}

//! Build script for BMW Diagnostic Daemon (Debian/Linux only)
//!
//! Handles FTDI D2XX library linking on Linux.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Check if libftd2xx is installed on Linux
    let lib_paths = vec![
        "/usr/local/lib/libftd2xx.so",
        "/usr/lib/libftd2xx.so",
        "/usr/lib/x86_64-linux-gnu/libftd2xx.so",
    ];

    let found = lib_paths.iter().any(|p| std::path::Path::new(p).exists());

    if !found {
        println!("cargo:warning==============================================");
        println!("cargo:warning=libftd2xx.so not found!");
        println!("cargo:warning=");
        println!("cargo:warning=Install on Debian/Ubuntu:");
        println!("cargo:warning=  1. Download from: https://ftdichip.com/drivers/d2xx-drivers/");
        println!("cargo:warning=  2. Extract and copy libftd2xx.so to /usr/local/lib/");
        println!("cargo:warning=  3. Run: sudo ldconfig");
        println!("cargo:warning==============================================");
    }
}

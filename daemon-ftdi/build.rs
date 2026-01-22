//! Build script for BMW Diagnostic Daemon
//!
//! Handles platform-specific build configuration, particularly for Windows
//! FTDI D2XX library linking.

fn main() {
    // Windows-specific: Configure FTDI D2XX library linking
    #[cfg(target_os = "windows")]
    {
        // Get FTD2XX_DIR from environment or use default
        let ftdi_dir = std::env::var("FTD2XX_DIR").unwrap_or_else(|_| {
            // Check common locations
            let common_paths = vec![
                "C:\\ftdi",
                "C:\\Program Files\\FTDI\\D2XX",
                "C:\\Program Files (x86)\\FTDI\\D2XX",
            ];

            for path in &common_paths {
                if std::path::Path::new(path).exists() {
                    println!("cargo:warning=FTD2XX_DIR not set, using default: {}", path);
                    return path.to_string();
                }
            }

            // No default found, print helpful error
            println!("cargo:warning==============================================");
            println!("cargo:warning=FTDI D2XX library not found!");
            println!("cargo:warning=");
            println!("cargo:warning=Please set FTD2XX_DIR environment variable:");
            println!("cargo:warning=  $env:FTD2XX_DIR = \"C:\\ftdi\"");
            println!("cargo:warning=");
            println!("cargo:warning=Or download D2XX from:");
            println!("cargo:warning=  https://ftdichip.com/drivers/d2xx-drivers/");
            println!("cargo:warning==============================================");

            "C:\\ftdi".to_string()
        });

        // Tell cargo where to find the library
        println!("cargo:rustc-link-search=native={}", ftdi_dir);

        // Link against ftd2xx
        // Note: libftd2xx crate handles the actual linking, but we set up the path
        println!("cargo:rerun-if-env-changed=FTD2XX_DIR");

        // Check for library files
        let lib_path = std::path::Path::new(&ftdi_dir);

        // Check for 64-bit library
        let lib64 = lib_path.join("ftd2xx64.lib");
        let lib32 = lib_path.join("ftd2xx.lib");

        if !lib64.exists() && !lib32.exists() {
            println!("cargo:warning=No ftd2xx library files found in {}", ftdi_dir);
            println!("cargo:warning=Expected: ftd2xx64.lib or ftd2xx.lib");
        } else {
            if lib64.exists() {
                println!("cargo:warning=Found: ftd2xx64.lib (64-bit)");
            }
            if lib32.exists() {
                println!("cargo:warning=Found: ftd2xx.lib (32-bit)");
            }
        }
    }

    // Linux-specific: Check for libftd2xx
    #[cfg(target_os = "linux")]
    {
        // libftd2xx on Linux typically installs to /usr/local/lib
        // The libftd2xx crate handles this, but we can add checks
        println!("cargo:rerun-if-changed=build.rs");

        // Check if libftd2xx is installed
        let lib_paths = vec![
            "/usr/local/lib/libftd2xx.so",
            "/usr/lib/libftd2xx.so",
        ];

        let found = lib_paths.iter().any(|p| std::path::Path::new(p).exists());

        if !found {
            println!("cargo:warning=libftd2xx.so not found in standard locations");
            println!("cargo:warning=Install from: https://ftdichip.com/drivers/d2xx-drivers/");
        }
    }
}

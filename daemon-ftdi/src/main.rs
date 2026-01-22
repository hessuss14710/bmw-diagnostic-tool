//! BMW Diagnostic Daemon - High Precision FTDI Control
//!
//! This daemon provides microsecond-level timing control for K-Line
//! communication with BMW ECUs using FTDI D2XX direct drivers.

mod ftdi;
mod kline;
mod kwp2000;
mod websocket;

use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .compact()
        .init();

    println!("╔═══════════════════════════════════════════════════════╗");
    println!("║     BMW Diagnostic Daemon v1.0 - FTDI D2XX            ║");
    println!("║     High-Precision K-Line Communication               ║");
    println!("╚═══════════════════════════════════════════════════════╝");
    println!();

    // List available FTDI devices
    info!("Scanning for FTDI devices...");
    let devices = ftdi::list_devices()?;

    if devices.is_empty() {
        println!("⚠️  No FTDI devices found!");
        println!("   Make sure your K+DCAN cable is connected.");
        println!("   Install FTDI D2XX drivers from: https://ftdichip.com/drivers/d2xx-drivers/");
        return Ok(());
    }

    println!("Found {} FTDI device(s):", devices.len());
    for (i, dev) in devices.iter().enumerate() {
        println!("  [{}] {} - {}", i, dev.description, dev.serial_number);
    }
    println!();

    // Start WebSocket server
    let port = 3003;
    info!("Starting WebSocket server on port {}...", port);

    websocket::run_server(port).await?;

    Ok(())
}

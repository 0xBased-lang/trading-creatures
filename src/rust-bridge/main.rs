// Main entry point for Trading Bridge Server

mod trading_bridge;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧬 Trading Creatures - Signal Validation Bridge");
    println!("================================================\n");

    trading_bridge::start_server().await?;

    Ok(())
}

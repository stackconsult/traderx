//! Engineering Agent Orchestra Q&A System Binary
//! 
//! Demonstrates deterministic multi-agent Q&A processing

use oms_engine::engineering_orchestra::demonstrate_orchestra;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    demonstrate_orchestra().await
}

//! Write-Ahead Logging for crash recovery.
//! Reuses the Aeron journal infrastructure for durability.

use crate::engine::{AggregatorEvent, FillEvent, PriceUpdate};
use anyhow::{Context, Result};
use serde_json;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::Path;
use tokio::fs;
use tracing::{debug, error, info};

pub struct WAL {
    file_path: String,
}

impl WAL {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_owned(),
        }
    }

    /// Log a fill event to the WAL.
    pub fn log_fill(&self, fill: &FillEvent) -> Result<()> {
        let event = AggregatorEvent::Fill(fill.clone());
        self.log_event(&event)
    }

    /// Log a price update to the WAL.
    pub fn log_price(&self, price: &PriceUpdate) -> Result<()> {
        let event = AggregatorEvent::Price(price.clone());
        self.log_event(&event)
    }

    /// Write an event to the WAL file.
    fn log_event(&self, event: &AggregatorEvent) -> Result<()> {
        let json = serde_json::to_string(event)
            .context("serialize WAL event")?;
        
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)
            .context("open WAL file")?;
        
        writeln!(file, "{}", json)
            .context("write WAL entry")?;
        
        file.sync_all()
            .context("sync WAL to disk")?;
        
        Ok(())
    }

    /// Read all events from the WAL file.
    pub async fn read_all(&self) -> Result<Vec<AggregatorEvent>> {
        if !Path::new(&self.file_path).exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.file_path)
            .await
            .context("read WAL file")?;

        let mut events = Vec::new();
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<AggregatorEvent>(line) {
                Ok(event) => events.push(event),
                Err(e) => {
                    error!("Failed to deserialize WAL entry: {} — {}", e, line);
                }
            }
        }

        info!("Read {} events from WAL", events.len());
        Ok(events)
    }

    /// Truncate the WAL file (call after successful checkpoint).
    pub fn truncate(&self) -> Result<()> {
        std::fs::write(&self.file_path, "")
            .context("truncate WAL")?;
        debug!("WAL truncated");
        Ok(())
    }
}

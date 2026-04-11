use anyhow::{Context, Result};
use crossbeam_channel::{bounded, Receiver, Sender};
use questdb::ingress::{Buffer, Sender as QdbSender, SenderBuilder, TimestampNanos};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Canonical tick representation — matches QuestDB `ticks` table schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tick {
    pub ts_nanos: i64,
    pub symbol: String,
    pub exchange: String,
    pub price: f64,
    pub volume: f64,
    pub side: String,
    pub bid: f64,
    pub ask: f64,
    pub bid_size: f64,
    pub ask_size: f64,
    pub sequence: i64,
}

/// OHLCV bar — matches `ohlcv_1m` / `ohlcv_1h` schemas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OhlcvBar {
    pub ts_nanos: i64,
    pub symbol: String,
    pub exchange: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub vwap: f64,
    pub trade_count: i64,
}

/// Events that can be written to QuestDB.
#[derive(Debug)]
pub enum IngestionEvent {
    Tick(Tick),
    Bar1m(OhlcvBar),
    Bar1h(OhlcvBar),
    Flush,
    Shutdown,
}

/// Configuration for the ILP writer.
#[derive(Debug, Clone)]
pub struct WriterConfig {
    pub host: String,
    pub port: u16,
    /// Maximum number of rows to buffer before auto-flushing.
    pub batch_size: usize,
    /// Maximum age of buffered data before forced flush.
    pub flush_interval_ms: u64,
    /// Channel capacity (backpressure).
    pub channel_capacity: usize,
}

impl Default for WriterConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 9009,
            batch_size: 10_000,
            flush_interval_ms: 50,
            channel_capacity: 1_000_000,
        }
    }
}

/// Handle returned to producers — clones are cheap (just channel clone).
#[derive(Clone)]
pub struct WriterHandle {
    tx: Sender<IngestionEvent>,
}

impl WriterHandle {
    /// Non-blocking send — returns `Err` if channel is full (backpressure signal).
    #[inline]
    pub fn send_tick(&self, tick: Tick) -> Result<(), Tick> {
        self.tx
            .try_send(IngestionEvent::Tick(tick))
            .map_err(|e| match e {
                crossbeam_channel::TrySendError::Full(IngestionEvent::Tick(t)) => t,
                _ => unreachable!(),
            })
    }

    /// Blocking send — use only from async tasks via `spawn_blocking`.
    #[inline]
    pub fn send_tick_blocking(&self, tick: Tick) {
        let _ = self.tx.send(IngestionEvent::Tick(tick));
    }

    pub fn flush(&self) {
        let _ = self.tx.send(IngestionEvent::Flush);
    }

    pub fn shutdown(&self) {
        let _ = self.tx.send(IngestionEvent::Shutdown);
    }
}

/// Background writer thread — owns the QuestDB TCP connection.
pub struct TickWriter {
    cfg: WriterConfig,
    rx: Receiver<IngestionEvent>,
}

impl TickWriter {
    /// Spawn writer on a dedicated OS thread. Returns a `WriterHandle`.
    pub fn spawn(cfg: WriterConfig) -> Result<WriterHandle> {
        let (tx, rx) = bounded(cfg.channel_capacity);
        let writer = TickWriter { cfg: cfg.clone(), rx };

        std::thread::Builder::new()
            .name("questdb-writer".into())
            .spawn(move || {
                if let Err(e) = writer.run() {
                    error!("QuestDB writer thread exited with error: {e:#}");
                }
            })
            .context("spawn questdb-writer thread")?;

        info!(
            "QuestDB ILP writer started → {}:{}",
            cfg.host, cfg.port
        );
        Ok(WriterHandle { tx })
    }

    fn run(self) -> Result<()> {
        let mut sender = SenderBuilder::new(&self.cfg.host, self.cfg.port)
            .build()
            .context("connect to QuestDB ILP")?;

        let mut buffer = Buffer::new();
        let mut buffered: usize = 0;
        let mut last_flush = Instant::now();
        let flush_interval = Duration::from_millis(self.cfg.flush_interval_ms);

        loop {
            // Drain available events without blocking, then check flush condition.
            match self.rx.recv_timeout(Duration::from_millis(5)) {
                Ok(event) => match event {
                    IngestionEvent::Tick(t) => {
                        Self::encode_tick(&mut buffer, &t)?;
                        buffered += 1;
                    }
                    IngestionEvent::Bar1m(b) => {
                        Self::encode_bar(&mut buffer, "ohlcv_1m", &b)?;
                        buffered += 1;
                    }
                    IngestionEvent::Bar1h(b) => {
                        Self::encode_bar(&mut buffer, "ohlcv_1h", &b)?;
                        buffered += 1;
                    }
                    IngestionEvent::Flush => {
                        Self::flush(&mut sender, &mut buffer, &mut buffered, &mut last_flush)?;
                        continue;
                    }
                    IngestionEvent::Shutdown => {
                        Self::flush(&mut sender, &mut buffer, &mut buffered, &mut last_flush)?;
                        info!("QuestDB writer: clean shutdown");
                        return Ok(());
                    }
                },
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {}
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                    warn!("QuestDB writer: channel disconnected, flushing and exiting");
                    Self::flush(&mut sender, &mut buffer, &mut buffered, &mut last_flush)?;
                    return Ok(());
                }
            }

            // Auto-flush on batch size or interval.
            if buffered >= self.cfg.batch_size
                || (buffered > 0 && last_flush.elapsed() >= flush_interval)
            {
                Self::flush(&mut sender, &mut buffer, &mut buffered, &mut last_flush)?;
            }
        }
    }

    fn encode_tick(buf: &mut Buffer, t: &Tick) -> Result<()> {
        buf.table("ticks")?
            .symbol("symbol", &t.symbol)?
            .symbol("exchange", &t.exchange)?
            .symbol("side", &t.side)?
            .column_f64("price", t.price)?
            .column_f64("volume", t.volume)?
            .column_f64("bid", t.bid)?
            .column_f64("ask", t.ask)?
            .column_f64("bid_size", t.bid_size)?
            .column_f64("ask_size", t.ask_size)?
            .column_i64("sequence", t.sequence)?
            .at(TimestampNanos::new(t.ts_nanos))?;
        Ok(())
    }

    fn encode_bar(buf: &mut Buffer, table: &str, b: &OhlcvBar) -> Result<()> {
        buf.table(table)?
            .symbol("symbol", &b.symbol)?
            .symbol("exchange", &b.exchange)?
            .column_f64("open", b.open)?
            .column_f64("high", b.high)?
            .column_f64("low", b.low)?
            .column_f64("close", b.close)?
            .column_f64("volume", b.volume)?
            .column_f64("vwap", b.vwap)?
            .column_i64("trade_count", b.trade_count)?
            .at(TimestampNanos::new(b.ts_nanos))?;
        Ok(())
    }

    fn flush(
        sender: &mut QdbSender,
        buffer: &mut Buffer,
        buffered: &mut usize,
        last_flush: &mut Instant,
    ) -> Result<()> {
        if *buffered == 0 {
            return Ok(());
        }
        sender.flush(buffer).context("QuestDB ILP flush")?;
        debug!("Flushed {} rows to QuestDB", buffered);
        *buffered = 0;
        *last_flush = Instant::now();
        Ok(())
    }
}

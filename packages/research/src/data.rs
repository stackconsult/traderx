use crate::engine::Bar;
use anyhow::{Context, Result};

/// Load OHLCV bars from a Parquet file (written by feature-store pipeline).
/// Uses Polars for zero-copy columnar read.
pub fn load_parquet(path: &str, symbol: &str) -> Result<Vec<Bar>> {
    use polars::prelude::*;

    let df = LazyFrame::scan_parquet(path, ScanArgsParquet::default())
        .context("open parquet")?
        .filter(col("symbol").eq(lit(symbol)))
        .select([
            col("ts"),
            col("open"), col("high"), col("low"), col("close"),
            col("volume"), col("vwap"),
        ])
        .sort("ts", SortOptions::default())
        .collect()
        .context("collect parquet")?;

    bars_from_df(&df, symbol)
}

/// Load from CSV (fallback / testing).
pub fn load_csv(path: &str, symbol: &str) -> Result<Vec<Bar>> {
    use polars::prelude::*;
    let df = CsvReadOptions::default()
        .with_has_header(true)
        .try_into_reader_with_file_path(Some(path.into()))
        .context("open csv")?
        .finish()
        .context("read csv")?;
    bars_from_df(&df, symbol)
}

/// Load bars directly from QuestDB via HTTP REST API.
pub async fn load_from_questdb(
    host: &str,
    http_port: u16,
    symbol: &str,
    limit: usize,
) -> Result<Vec<Bar>> {
    let query = format!(
        "SELECT ts, open, high, low, close, volume, vwap \
         FROM ohlcv_1m WHERE symbol='{}' ORDER BY ts DESC LIMIT {}",
        symbol, limit
    );
    let url = format!(
        "http://{}:{}/exec?query={}&fmt=json",
        host, http_port,
        urlencoding::encode(&query)
    );
    let resp: serde_json::Value = reqwest::get(&url)
        .await.context("GET questdb")?
        .json().await.context("parse json")?;

    let dataset = resp["dataset"].as_array().context("no dataset")?;
    let mut bars = Vec::with_capacity(dataset.len());
    for row in dataset.iter().rev() {
        let arr = row.as_array().context("row not array")?;
        bars.push(Bar {
            ts_nanos: arr[0].as_i64().unwrap_or(0),
            symbol:   symbol.to_owned(),
            open:     arr[1].as_f64().unwrap_or(0.0),
            high:     arr[2].as_f64().unwrap_or(0.0),
            low:      arr[3].as_f64().unwrap_or(0.0),
            close:    arr[4].as_f64().unwrap_or(0.0),
            volume:   arr[5].as_f64().unwrap_or(0.0),
            vwap:     arr[6].as_f64().unwrap_or(0.0),
        });
    }
    Ok(bars)
}

fn bars_from_df(df: &polars::frame::DataFrame, symbol: &str) -> Result<Vec<Bar>> {
    use polars::prelude::*;
    let n = df.height();
    let ts     = df.column("ts")?.i64()?;
    let open   = df.column("open")?.f64()?;
    let high   = df.column("high")?.f64()?;
    let low    = df.column("low")?.f64()?;
    let close  = df.column("close")?.f64()?;
    let volume = df.column("volume")?.f64()?;
    let vwap   = df.column("vwap")?.f64()?;

    let mut bars = Vec::with_capacity(n);
    for i in 0..n {
        bars.push(Bar {
            ts_nanos: ts.get(i).unwrap_or(0),
            symbol:   symbol.to_owned(),
            open:     open.get(i).unwrap_or(0.0),
            high:     high.get(i).unwrap_or(0.0),
            low:      low.get(i).unwrap_or(0.0),
            close:    close.get(i).unwrap_or(0.0),
            volume:   volume.get(i).unwrap_or(0.0),
            vwap:     vwap.get(i).unwrap_or(0.0),
        });
    }
    Ok(bars)
}

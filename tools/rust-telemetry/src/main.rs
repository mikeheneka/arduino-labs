use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use anyhow::Result;
use clap::Parser;
use csv::Writer;
use ctrlc;
use rusqlite::{Connection, params};

use rust_telemetry::telemetry::{self, DEFAULT_PORT, Record, SerialStreamConfig};

#[derive(Parser, Debug)]
#[command(author, version, about = "Rust companion for the serial-telemetry sketch", long_about = None)]
struct Cli {
    /// Serial port device path (e.g., /dev/cu.usbserial-10 or COM4)
    #[arg(short, long, default_value = DEFAULT_PORT)]
    port: String,

    /// Baud rate configured in the Arduino sketch
    #[arg(short, long, default_value_t = 115_200)]
    baud: u32,

    /// Optional CSV file to append readings to
    #[arg(long, value_name = "PATH")]
    csv: Option<PathBuf>,

    /// Optional SQLite database file to persist readings
    #[arg(long, value_name = "PATH")]
    sqlite: Option<PathBuf>,

    /// List detected serial ports and exit
    #[arg(long)]
    list_ports: bool,

    /// Read timeout in milliseconds
    #[arg(long, default_value_t = 1000)]
    timeout_ms: u64,
}

struct CsvSink {
    writer: Writer<std::fs::File>,
}

impl CsvSink {
    fn new(path: &Path) -> Result<Self> {
        let needs_header = !path.exists() || path.metadata()?.len() == 0;
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        let mut writer = Writer::from_writer(file);
        if needs_header {
            writer.write_record(["timestamp", "raw", "voltage"])?;
            writer.flush()?;
        }
        Ok(Self { writer })
    }

    fn write(&mut self, record: &Record) -> Result<()> {
        self.writer.serialize(record)?;
        self.writer.flush()?;
        Ok(())
    }
}

struct SqliteSink {
    conn: Connection,
}

impl SqliteSink {
    fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS readings (\n                timestamp TEXT NOT NULL,\n                raw INTEGER NOT NULL,\n                voltage REAL NOT NULL\n            )",
            [],
        )?;
        Ok(Self { conn })
    }

    fn write(&self, record: &Record) -> Result<()> {
        self.conn.execute(
            "INSERT INTO readings (timestamp, raw, voltage) VALUES (?1, ?2, ?3)",
            params![record.timestamp.to_rfc3339(), record.raw, record.voltage],
        )?;
        Ok(())
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.list_ports {
        return telemetry::list_ports();
    }

    let mut csv_sink = if let Some(path) = cli.csv.as_ref() {
        Some(CsvSink::new(path)?)
    } else {
        None
    };
    let sqlite_sink = if let Some(path) = cli.sqlite.as_ref() {
        Some(SqliteSink::new(path)?)
    } else {
        None
    };

    if csv_sink.is_none() && sqlite_sink.is_none() {
        println!(
            "⚠️  No persistence flags supplied. Pass --csv <file> and/or --sqlite <db> to log readings."
        );
    }

    let config = SerialStreamConfig {
        port: cli.port.clone(),
        baud: cli.baud,
        timeout_ms: cli.timeout_ms,
    };

    let shutdown = Arc::new(AtomicBool::new(false));
    let signal_flag = shutdown.clone();
    ctrlc::set_handler(move || {
        signal_flag.store(true, Ordering::SeqCst);
    })?;

    telemetry::stream_records(config, shutdown.clone(), |record| {
        println!(
            "{} | raw: {:>4} | voltage: {:.3} V",
            record.timestamp.to_rfc3339(),
            record.raw,
            record.voltage
        );

        if let Some(writer) = csv_sink.as_mut() {
            writer.write(&record)?;
        }
        if let Some(conn) = sqlite_sink.as_ref() {
            conn.write(&record)?;
        }
        Ok(())
    })?;

    println!("Signal received, closing port.");
    Ok(())
}

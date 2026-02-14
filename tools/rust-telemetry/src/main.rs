use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use clap::Parser;
use csv::Writer;
use ctrlc;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

const DEFAULT_PORT: &str = "/dev/cu.usbserial-10";

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

#[derive(Debug, Deserialize)]
struct TelemetryReading {
    raw: i32,
    voltage: f32,
}

#[derive(Debug, Serialize)]
struct Record {
    timestamp: DateTime<Utc>,
    raw: i32,
    voltage: f32,
}

struct CsvSink {
    writer: Writer<std::fs::File>,
}

impl CsvSink {
    fn new(path: &Path) -> Result<Self> {
        let needs_header = !path.exists() || path.metadata()?.len() == 0;
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .with_context(|| format!("failed to open CSV log at {}", path.display()))?;
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
        let conn = Connection::open(path)
            .with_context(|| format!("failed to open sqlite db at {}", path.display()))?;
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

fn list_ports() -> Result<()> {
    let ports = serialport::available_ports().context("unable to enumerate serial ports")?;
    if ports.is_empty() {
        println!("No serial ports detected.");
        return Ok(());
    }

    println!("Detected serial ports:\n----------------------");
    for port in ports {
        print!("{}", port.port_name);
        if let Some(desc) = describe_port_type(&port.port_type) {
            print!(" — {}", desc);
        }
        println!();
    }
    Ok(())
}

fn describe_port_type(port_type: &serialport::SerialPortType) -> Option<String> {
    match port_type {
        serialport::SerialPortType::UsbPort(info) => Some(format!(
            "USB VID:PID {:04x}:{:04x} {}",
            info.vid,
            info.pid,
            info.serial_number
                .as_ref()
                .map(|s| format!("(serial {s})"))
                .unwrap_or_default()
        )),
        serialport::SerialPortType::PciPort => Some("PCI".to_string()),
        serialport::SerialPortType::BluetoothPort => Some("Bluetooth".to_string()),
        serialport::SerialPortType::Unknown => None,
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.list_ports {
        return list_ports();
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

    println!(
        "Opening {} @ {} baud (timeout {} ms)…",
        cli.port, cli.baud, cli.timeout_ms
    );

    let port = serialport::new(&cli.port, cli.baud)
        .timeout(Duration::from_millis(cli.timeout_ms))
        .open()
        .with_context(|| format!("failed to open serial port {}", cli.port))?;

    let mut reader = BufReader::new(port);
    let shutdown = Arc::new(AtomicBool::new(false));
    let signal_flag = shutdown.clone();
    ctrlc::set_handler(move || {
        signal_flag.store(true, Ordering::SeqCst);
    })?;

    let mut line = String::new();
    while !shutdown.load(Ordering::SeqCst) {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => continue,
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                match serde_json::from_str::<TelemetryReading>(trimmed) {
                    Ok(parsed) => {
                        let record = Record {
                            timestamp: Utc::now(),
                            raw: parsed.raw,
                            voltage: parsed.voltage,
                        };
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
                    }
                    Err(err) => eprintln!("Failed to parse line `{trimmed}`: {err}"),
                }
            }
            Err(e) if e.kind() == io::ErrorKind::TimedOut => continue,
            Err(e) => return Err(anyhow!("serial read error: {e}")),
        }
    }

    println!("Signal received, closing port.");
    Ok(())
}

use std::io::{self, BufRead, BufReader};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serialport::SerialPort;

pub const DEFAULT_PORT: &str = "/dev/cu.usbserial-10";

#[derive(Debug, Deserialize, Clone)]
pub struct TelemetryReading {
    pub raw: i32,
    pub voltage: f32,
}

#[derive(Debug, Serialize, Clone)]
pub struct Record {
    pub timestamp: DateTime<Utc>,
    pub raw: i32,
    pub voltage: f32,
}

#[derive(Clone)]
pub struct SerialStreamConfig {
    pub port: String,
    pub baud: u32,
    pub timeout_ms: u64,
}

impl SerialStreamConfig {
    pub fn open_port(&self) -> Result<Box<dyn SerialPort>> {
        serialport::new(&self.port, self.baud)
            .timeout(Duration::from_millis(self.timeout_ms))
            .open()
            .with_context(|| format!("failed to open serial port {}", self.port))
    }
}

pub fn list_ports() -> Result<()> {
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

pub fn stream_records<F>(
    config: SerialStreamConfig,
    shutdown: Arc<AtomicBool>,
    mut handler: F,
) -> Result<()>
where
    F: FnMut(Record) -> Result<()>,
{
    println!(
        "Opening {} @ {} baud (timeout {} ms)…",
        config.port, config.baud, config.timeout_ms
    );

    let port = config.open_port()?;
    let mut reader = BufReader::new(port);
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
                        handler(record)?;
                    }
                    Err(err) => eprintln!("Failed to parse line `{trimmed}`: {err}"),
                }
            }
            Err(e) if e.kind() == io::ErrorKind::TimedOut => continue,
            Err(e) => return Err(anyhow!("serial read error: {e}")),
        }
    }

    Ok(())
}

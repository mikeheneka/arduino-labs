use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread;

use anyhow::{Context, Result};
use axum::http::StatusCode;
use axum::{
    Json, Router,
    extract::State,
    response::{Html, IntoResponse},
    routing::get,
};
use clap::Parser;
use rust_telemetry::telemetry::{self, DEFAULT_PORT, Record, SerialStreamConfig};
use tokio::signal;

#[derive(Parser, Debug)]
#[command(author, version, about = "Axum dashboard for Arduino telemetry")]
struct DashboardCli {
    /// Serial port device path (e.g., /dev/cu.usbserial-10 or COM4)
    #[arg(short, long, default_value = DEFAULT_PORT)]
    port: String,

    /// Baud rate configured in the Arduino sketch
    #[arg(short, long, default_value_t = 115_200)]
    baud: u32,

    /// Read timeout in milliseconds
    #[arg(long, default_value_t = 1000)]
    timeout_ms: u64,

    /// Bind address for the HTTP server
    #[arg(long, default_value = "127.0.0.1:7878")]
    bind: String,

    /// Number of recent samples to keep in memory
    #[arg(long, default_value_t = 100)]
    window: usize,

    /// List detected serial ports and exit
    #[arg(long)]
    list_ports: bool,
}

#[derive(Clone)]
struct AppState {
    history: Arc<Mutex<VecDeque<Record>>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = DashboardCli::parse();

    if cli.list_ports {
        return telemetry::list_ports();
    }

    let addr: SocketAddr = cli
        .bind
        .parse()
        .with_context(|| format!("failed to parse bind address {}", cli.bind))?;

    let history = Arc::new(Mutex::new(VecDeque::with_capacity(cli.window)));
    let state = AppState {
        history: history.clone(),
    };

    let shutdown = Arc::new(AtomicBool::new(false));
    let reader_shutdown = shutdown.clone();
    let reader_config = SerialStreamConfig {
        port: cli.port.clone(),
        baud: cli.baud,
        timeout_ms: cli.timeout_ms,
    };
    let window = cli.window;

    let reader_handle = thread::spawn(move || {
        if let Err(err) =
            telemetry::stream_records(reader_config, reader_shutdown.clone(), |record| {
                {
                    let mut hist = history.lock().expect("history mutex poisoned");
                    if hist.len() == window {
                        hist.pop_front();
                    }
                    hist.push_back(record.clone());
                }
                println!(
                    "{} | raw: {:>4} | voltage: {:.3} V",
                    record.timestamp.to_rfc3339(),
                    record.raw,
                    record.voltage
                );
                Ok(())
            })
        {
            eprintln!("Serial stream terminated: {err:?}");
        }
    });

    let app = Router::new()
        .route("/", get(index))
        .route("/api/latest", get(latest))
        .route("/api/history", get(history_handler))
        .with_state(state.clone());

    println!("Dashboard listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    let graceful_shutdown = shutdown.clone();
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(async move {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
            println!("Shutdown requested, closing serial reader…");
            graceful_shutdown.store(true, Ordering::SeqCst);
        })
        .await?;

    shutdown.store(true, Ordering::SeqCst);
    reader_handle.join().ok();

    Ok(())
}

async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn latest(State(state): State<AppState>) -> impl IntoResponse {
    let record = state.history.lock().unwrap().back().cloned();
    match record {
        Some(rec) => Json(rec).into_response(),
        None => StatusCode::NO_CONTENT.into_response(),
    }
}

async fn history_handler(State(state): State<AppState>) -> Json<Vec<Record>> {
    let data: Vec<Record> = state.history.lock().unwrap().iter().cloned().collect();
    Json(data)
}

const INDEX_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8" />
<title>Arduino Telemetry Dashboard</title>
<meta name="viewport" content="width=device-width, initial-scale=1" />
<link rel="preconnect" href="https://fonts.googleapis.com" />
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
<link href="https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@400;500;600&display=swap" rel="stylesheet" />
<script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
<style>
:root {
  color-scheme: dark;
  --bg: radial-gradient(circle at top, #182848, #0b1221 55%);
  --card: rgba(7, 16, 34, 0.92);
  --accent: #72e0ff;
  --muted: #97a4ce;
}
html, body {
  margin: 0;
  padding: 0;
  min-height: 100%;
  font-family: 'Space Grotesk', system-ui, sans-serif;
  background: var(--bg);
  color: #f1f5ff;
}
main {
  max-width: 1100px;
  margin: 0 auto;
  padding: 2.5rem 1.5rem 3rem;
}
.card {
  background: var(--card);
  border-radius: 1.5rem;
  padding: 2rem;
  box-shadow: 0 35px 80px rgba(3, 5, 20, 0.65);
  backdrop-filter: blur(12px);
}
h1 {
  margin: 0;
  font-size: clamp(1.9rem, 3vw, 2.6rem);
}
section + section {
  margin-top: 1.75rem;
}
.metrics {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
  gap: 1rem;
}
.metric {
  background: rgba(255, 255, 255, 0.04);
  border-radius: 1rem;
  padding: 1rem 1.1rem;
  border: 1px solid rgba(255, 255, 255, 0.06);
  min-height: 94px;
}
.metric span {
  display: block;
  font-size: 0.78rem;
  text-transform: uppercase;
  letter-spacing: 0.09em;
  color: var(--muted);
}
.metric strong {
  display: block;
  margin-top: 0.35rem;
  font-size: 1.55rem;
  font-weight: 600;
  color: var(--accent);
  word-break: break-word;
}
.metric.small strong {
  font-size: 1.2rem;
}
#history {
  background: rgba(0, 0, 0, 0.35);
  border-radius: 1rem;
  padding: 1rem;
  font-family: 'JetBrains Mono', 'SFMono-Regular', ui-monospace, monospace;
  font-size: 0.86rem;
  max-height: 320px;
  overflow-y: auto;
  border: 1px solid rgba(255, 255, 255, 0.05);
}
.chart-card {
  padding: 1.2rem;
  background: rgba(255, 255, 255, 0.02);
  border-radius: 1rem;
  border: 1px solid rgba(255, 255, 255, 0.05);
}
</style>
</head>
<body>
<main>
  <div class="card">
    <h1>Arduino Telemetry</h1>
    <section>
      <div class="metrics">
        <div class="metric"><span>Voltage</span><strong id="voltage">—</strong></div>
        <div class="metric"><span>Raw</span><strong id="raw">—</strong></div>
        <div class="metric"><span>Supply (Vcc)</span><strong id="supply">—</strong></div>
        <div class="metric"><span>Timestamp</span><strong id="timestamp">—</strong></div>
      </div>
    </section>
    <section>
      <h2>Device Health</h2>
      <div class="metrics">
        <div class="metric"><span>Button Presses</span><strong id="button-count">—</strong></div>
        <div class="metric"><span>Last Button</span><strong id="button-last">—</strong></div>
        <div class="metric"><span>Loop Duration</span><strong id="loop">—</strong></div>
        <div class="metric"><span>Uptime</span><strong id="uptime">—</strong></div>
        <div class="metric small"><span>Firmware</span><strong id="firmware">—</strong></div>
      </div>
    </section>
    <section>
      <h2>Live Signals</h2>
      <div class="chart-card">
        <canvas id="voltage-chart" height="120"></canvas>
      </div>
    </section>
    <section>
      <h2>Recent History</h2>
      <pre id="history">Loading…</pre>
    </section>
  </div>
</main>
<script>
const voltagePoints = [];
const vccPoints = [];
let telemetryChart = null;

function formatDuration(ms) {
  if (ms == null) return '—';
  const totalSeconds = Math.floor(ms / 1000);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  return `${hours}h ${minutes}m ${seconds}s`;
}

function ensureChart() {
  if (telemetryChart) return telemetryChart;
  const ctx = document.getElementById('voltage-chart');
  telemetryChart = new Chart(ctx, {
    type: 'line',
    data: {
      labels: [],
      datasets: [
        {
          label: 'Voltage',
          data: [],
          tension: 0.25,
          borderColor: '#72e0ff',
          backgroundColor: 'rgba(114, 224, 255, 0.15)',
          fill: true,
          pointRadius: 0,
          borderWidth: 2,
        },
        {
          label: 'Vcc',
          data: [],
          tension: 0.25,
          borderColor: '#ffb86c',
          backgroundColor: 'rgba(255, 184, 108, 0.08)',
          fill: true,
          pointRadius: 0,
          borderWidth: 2,
        },
      ],
    },
    options: {
      plugins: {
        legend: {
          labels: { color: '#c8d4ff' },
        },
      },
      scales: {
        x: {
          ticks: { color: '#6f7ca6' },
          grid: { color: 'rgba(255,255,255,0.04)' },
        },
        y: {
          ticks: { color: '#6f7ca6' },
          grid: { color: 'rgba(255,255,255,0.04)' },
        },
      },
    },
  });
  return telemetryChart;
}

function pushPoint(buffer, value) {
  const limit = 60;
  buffer.push(value);
  if (buffer.length > limit) buffer.shift();
}

async function refresh() {
  const latest = await fetch('/api/latest');
  if (latest.status === 204) {
    document.getElementById('history').textContent = 'Waiting for serial data…';
    return;
  }
  const latestJson = await latest.json();
  document.getElementById('voltage').textContent = `${latestJson.voltage.toFixed(3)} V`;
  document.getElementById('raw').textContent = latestJson.raw;
  document.getElementById('timestamp').textContent = new Date(latestJson.timestamp).toLocaleTimeString();
  document.getElementById('supply').textContent = latestJson.vcc != null ? `${latestJson.vcc.toFixed(3)} V` : '—';
  document.getElementById('loop').textContent = latestJson.loop_ms != null ? `${latestJson.loop_ms.toFixed(2)} ms` : '—';
  document.getElementById('uptime').textContent = formatDuration(latestJson.uptime_ms);
  document.getElementById('firmware').textContent = latestJson.firmware ?? '—';
  const button = latestJson.button ?? {};
  document.getElementById('button-count').textContent = button.count ?? 0;
  const delta = button.last_press_delta_ms;
  document.getElementById('button-last').textContent = (delta == null || delta < 0)
    ? 'never'
    : `${(delta / 1000).toFixed(1)} s ago`;

  const historyResp = await fetch('/api/history');
  const historyJson = await historyResp.json();
  document.getElementById('history').textContent = historyJson
    .slice(-25)
    .map((item) => {
      const supply = item.vcc != null ? `${item.vcc.toFixed(3)}V` : '—';
      const btn = item.button?.count ?? 0;
      return `${item.timestamp} raw=${item.raw} voltage=${item.voltage.toFixed(3)}V vcc=${supply} button=${btn}`;
    })
    .join('\n');

  const chart = ensureChart();
  const label = new Date(latestJson.timestamp).toLocaleTimeString();
  pushPoint(voltagePoints, latestJson.voltage);
  pushPoint(vccPoints, latestJson.vcc ?? null);
  chart.data.labels.push(label);
  if (chart.data.labels.length > 60) chart.data.labels.shift();
  chart.data.datasets[0].data = [...voltagePoints];
  chart.data.datasets[1].data = vccPoints.map((v) => v ?? null);
  chart.update();
}
setInterval(refresh, 1500);
refresh();
</script>
</body>
</html>"#;

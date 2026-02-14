# Arduino Labs

[![Arduino CI](https://github.com/mikeheneka/arduino-labs/actions/workflows/arduino-ci.yml/badge.svg)](https://github.com/mikeheneka/arduino-labs/actions/workflows/arduino-ci.yml)

A collection of quick experiments targeting an Arduino Uno (ATmega328P + CH340).

## Projects

| Project | Summary |
| --- | --- |
| `blink-basic` | Onboard LED, 5s on / 1s off pattern |
| `pwm-fade` | PWM sweep on D9 for external LED/MOSFET control |
| `serial-telemetry` | A0 sampling + button/supply/uptime telemetry over JSON |
| `button-alert` | Interrupt on D2, flashes LED + emits event |
| `tools/rust-telemetry` | Rust CLI/Axum dashboard that captures and visualizes the telemetry feed |

Each project folder contains:

- `.ino` sketch
- Wiring notes / BOM
- Instructions for `arduino-cli`

### Serial telemetry quick notes

- **Analog channel**: `A0` is still the primary voltage input (expecting 0–5 V). The sketch now also emits the Uno’s measured Vcc so you can see supply droop without extra sensors.
- **Button telemetry**: `D2` is configured with `INPUT_PULLUP`. Touching it to GND (use a jumper or momentary switch) increments the press counter and timestamps the event that shows up in the dashboard/API.
- **Device health**: Every sample includes `uptime_ms`, loop duration, firmware version string, and last-button delta so downstream tools know if the firmware is wedged.

## Tooling

This repo assumes `arduino-cli` is installed and configured with the AVR core:

```bash
arduino-cli core install arduino:avr
```

Compile/upload example:

```bash
arduino-cli compile --fqbn arduino:avr:uno blink-basic
arduino-cli upload -p /dev/cu.usbserial-10 --fqbn arduino:avr:uno blink-basic
```

### Rust companion (`tools/rust-telemetry`)

Install Rust via `rustup` if you haven’t already, then:

```bash
cd tools/rust-telemetry
cargo run -- --list-ports
cargo run -- --port /dev/cu.usbserial-10 --csv telemetry.csv --sqlite telemetry.db
cargo run --bin dashboard -- --port /dev/cu.usbserial-10 --bind 0.0.0.0:7878 --window 200
```

Flags (collector CLI):

- `--port` (default `/dev/cu.usbserial-10`): serial device exposed by the Uno/CH340
- `--baud` (default `115200`): must match the sketch
- `--csv <file>`: append structured rows with timestamp/raw/voltage
- `--sqlite <file>`: maintain a `readings` table for downstream dashboards
- `--list-ports`: enumerate detected serial devices and exit

Dashboard-specific flags:

- `--bind` (default `127.0.0.1:7878`): address for the Axum HTTP server
- `--window` (default `100`): number of recent samples to retain/serve via the API

The CLI streams the JSON lines emitted by `serial-telemetry`, prints them with timestamps, and optionally persists them for later plotting or ingestion. The dashboard binary reuses the same serial stream, exposes `/api/latest`, `/api/history`, and serves a lightweight HTML card that now shows voltage, raw counts, Vcc, button activity, loop timing, uptime, and firmware metadata in one place.

## Roadmap

1. Flesh out telemetry ingestion scripts for OpenClaw automations
2. Add sensor-specific demos (TMP36 temp, HC-SR04 distance, etc.)
3. Provide wiring diagrams / Fritzing exports per project

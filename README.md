# Arduino Labs

[![Arduino CI](https://github.com/mikeheneka/arduino-labs/actions/workflows/arduino-ci.yml/badge.svg)](https://github.com/mikeheneka/arduino-labs/actions/workflows/arduino-ci.yml)

A collection of quick experiments targeting an Arduino Uno (ATmega328P + CH340).

## Projects

| Project | Summary |
| --- | --- |
| `blink-basic` | Onboard LED, 5s on / 1s off pattern |
| `pwm-fade` | PWM sweep on D9 for external LED/MOSFET control |
| `serial-telemetry` | A0 sampling with JSON serial output |
| `button-alert` | Interrupt on D2, flashes LED + emits event |
| `tools/rust-telemetry` | Rust CLI that captures the serial telemetry sketch into CSV/SQLite |

Each project folder contains:

- `.ino` sketch
- Wiring notes / BOM
- Instructions for `arduino-cli`

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
```

Flags:

- `--port` (default `/dev/cu.usbserial-10`): serial device exposed by the Uno/CH340
- `--baud` (default `115200`): must match the sketch
- `--csv <file>`: append structured rows with timestamp/raw/voltage
- `--sqlite <file>`: maintain a `readings` table for downstream dashboards
- `--list-ports`: enumerate detected serial devices and exit

The CLI streams the JSON lines emitted by `serial-telemetry`, prints them with timestamps, and optionally persists them for later plotting or ingestion.

## Roadmap

1. Flesh out telemetry ingestion scripts for OpenClaw automations
2. Add sensor-specific demos (TMP36 temp, HC-SR04 distance, etc.)
3. Provide wiring diagrams / Fritzing exports per project

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

## Roadmap

1. Flesh out telemetry ingestion scripts for OpenClaw automations
2. Add sensor-specific demos (TMP36 temp, HC-SR04 distance, etc.)
3. Provide wiring diagrams / Fritzing exports per project

# Arduino Labs

A collection of quick experiments targeting an Arduino Uno (ATmega328P + CH340).

## Projects

- `blink-basic`: onboard LED with extended duty cycle (5s on / 1s off)
- `pwm-fade`: PWM dimming loop on digital pin 9 for an external LED or MOSFET gate
- `serial-telemetry`: streams analog A0 readings + supply voltage over serial at 1 Hz
- `button-alert`: interrupt-driven push-button detector publishing alert pulses on serial

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

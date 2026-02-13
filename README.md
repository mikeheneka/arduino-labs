# Arduino Labs

A collection of quick experiments targeting an Arduino Uno (ATmega328P + CH340).

## Projects

- `blink-basic`: reference LED blink (with custom duty cycles)
- Upcoming:
  - sensor polling + serial logging
  - relay / actuator control
  - addressable LED demo

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

1. Document each experiment with README + diagrams
2. Add scripts to gather serial telemetry for automations
3. Integrate with local OpenClaw workflows for alerting/testing

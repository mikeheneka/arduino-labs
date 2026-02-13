# Serial Telemetry

- **Input:** Analog sensor on A0 (0-5V)
- **Output:** JSON line over serial@115200 every second

Use a TMP36, potentiometer, or any analog voltage divider.

## Compile
```bash
arduino-cli compile --fqbn arduino:avr:uno serial-telemetry
```

## Upload
```bash
arduino-cli upload -p /dev/cu.usbserial-10 --fqbn arduino:avr:uno serial-telemetry
```

# Button Alert

- **Button wiring:** momentary switch between D2 and GND (internal pull-up enabled)
- **Behavior:** interrupt on press, flashes onboard LED, emits JSON event over serial

## Compile
```bash
arduino-cli compile --fqbn arduino:avr:uno button-alert
```

## Upload
```bash
arduino-cli upload -p /dev/cu.usbserial-10 --fqbn arduino:avr:uno button-alert
```

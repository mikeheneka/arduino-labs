# PWM Fade

- **LED pin:** D9 (through 220Ω resistor)
- **Behavior:** fades brightness up/down continuously

## Compile
```bash
arduino-cli compile --fqbn arduino:avr:uno pwm-fade
```

## Upload
```bash
arduino-cli upload -p /dev/cu.usbserial-10 --fqbn arduino:avr:uno pwm-fade
```

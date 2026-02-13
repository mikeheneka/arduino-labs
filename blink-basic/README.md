# Blink (Extended Duty Cycle)

- **Hardware:** Arduino Uno R3 clone (ATmega328P + CH340)
- **LED:** onboard (digital pin 13 / `LED_BUILTIN`)
- **Pattern:** 5 seconds on, 1 second off

## Compile

```bash
arduino-cli compile --fqbn arduino:avr:uno blink-basic
```

## Upload

```bash
arduino-cli upload -p /dev/cu.usbserial-10 --fqbn arduino:avr:uno blink-basic
```

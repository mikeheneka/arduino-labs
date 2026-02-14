const uint8_t sensorPin = A0;
const uint8_t buttonPin = 2;  // uses the built-in pull-up and an external ground tap

volatile uint32_t buttonCount = 0;
volatile unsigned long lastButtonPressMs = 0;

void onButtonPress() {
  buttonCount++;
  lastButtonPressMs = millis();
}

float readVcc() {
#if defined(__AVR__)
  ADMUX = _BV(REFS0) | _BV(MUX3) | _BV(MUX2) | _BV(MUX1);
  delay(2);
  ADCSRA |= _BV(ADSC);
  while (bit_is_set(ADCSRA, ADSC)) {
    // wait for conversion
  }
  uint16_t raw = ADC;
  if (raw == 0) {
    return 0.0;
  }
  long millivolts = 1125300L / raw;  // 1.1V reference * 1023 * 1000
  return millivolts / 1000.0;
#else
  return 5.0;  // best effort fallback for non-AVR boards
#endif
}

void setup() {
  pinMode(sensorPin, INPUT);
  pinMode(buttonPin, INPUT_PULLUP);
  attachInterrupt(digitalPinToInterrupt(buttonPin), onButtonPress, FALLING);
  Serial.begin(115200);
}

void loop() {
  unsigned long loopStart = micros();

  int raw = analogRead(sensorPin);
  float voltage = (raw / 1023.0) * 5.0;
  float vcc = readVcc();
  unsigned long uptime = millis();

  noInterrupts();
  uint32_t count = buttonCount;
  unsigned long lastPress = lastButtonPressMs;
  interrupts();

  long lastPressDelta = (count == 0) ? -1 : (long)(uptime - lastPress);
  float loopMs = (micros() - loopStart) / 1000.0;

  Serial.print("{\"raw\":");
  Serial.print(raw);
  Serial.print(",\"voltage\":");
  Serial.print(voltage, 3);
  Serial.print(",\"vcc\":");
  Serial.print(vcc, 3);
  Serial.print(",\"uptime_ms\":");
  Serial.print(uptime);
  Serial.print(",\"loop_ms\":");
  Serial.print(loopMs, 2);
  Serial.print(",\"button\":{");
  Serial.print("\"count\":");
  Serial.print(count);
  Serial.print(",\"last_press_delta_ms\":");
  Serial.print(lastPressDelta);
  Serial.print("}");
  Serial.print(",\"firmware\":\"serial-telemetry@1.1\"");
  Serial.println("}");

  delay(1000);
}

const int ledPin = LED_BUILTIN;

void setup() {
  pinMode(ledPin, OUTPUT);
}

void loop() {
  digitalWrite(ledPin, HIGH);
  delay(5000); // 5 seconds on
  digitalWrite(ledPin, LOW);
  delay(1000); // 1 second off
}

const int sensorPin = A0;

void setup() {
  Serial.begin(115200);
}

void loop() {
  int raw = analogRead(sensorPin);
  float voltage = (raw / 1023.0) * 5.0;
  Serial.print("{\"raw\":");
  Serial.print(raw);
  Serial.print(",\"voltage\":");
  Serial.print(voltage, 3);
  Serial.println("}");
  delay(1000);
}

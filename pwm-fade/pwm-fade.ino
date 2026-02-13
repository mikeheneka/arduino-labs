const int pwmPin = 9; // PWM-capable pin

void setup() {
  pinMode(pwmPin, OUTPUT);
}

void loop() {
  for (int duty = 0; duty <= 255; duty++) {
    analogWrite(pwmPin, duty);
    delay(10);
  }
  for (int duty = 255; duty >= 0; duty--) {
    analogWrite(pwmPin, duty);
    delay(10);
  }
}

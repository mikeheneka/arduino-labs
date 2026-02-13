const int buttonPin = 2; // interrupt-capable
const int ledPin = 13;
volatile bool alert = false;

void setup() {
  pinMode(buttonPin, INPUT_PULLUP);
  pinMode(ledPin, OUTPUT);
  attachInterrupt(digitalPinToInterrupt(buttonPin), onPress, FALLING);
  Serial.begin(115200);
}

void loop() {
  if (alert) {
    alert = false;
    digitalWrite(ledPin, HIGH);
    Serial.println("{\"event\":\"button_press\"}");
    delay(200);
    digitalWrite(ledPin, LOW);
  }
}

void onPress() {
  alert = true;
}

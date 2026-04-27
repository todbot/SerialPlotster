/**
 * test_three_channels.ino
 *
 * Outputs three labelled channels at 100 Hz in SerialPlotster format:
 *   sin:0.9511,cos:0.3090,noise:0.1234
 *
 * The noise signal is a deterministic pseudo-noise (two mixed sinusoids)
 * that matches the SerialPlotster built-in mock stream.
 *
 * Compatible with any Arduino board at 115200 baud.
 */

static const int    RATE_HZ     = 100;
static const float  DT          = 1.0f / RATE_HZ;
static const unsigned long INTERVAL_MS = 1000UL / RATE_HZ;

static float          t      = 0.0f;
static unsigned long  lastMs = 0;

void setup() {
  Serial.begin(115200);
  while (!Serial) { /* wait for USB CDC on Leonardo / 32u4 boards */ }
}

void loop() {
  unsigned long now = millis();
  if (now - lastMs < INTERVAL_MS) return;
  lastMs += INTERVAL_MS;   // drift-free: advance by fixed step, not now

  float s     = sinf(t);
  float c     = cosf(t);
  float noise = (sinf(t * 7.31f) * 0.7f + cosf(t * 3.17f) * 0.3f) * 0.5f;

  Serial.print("sin:");   Serial.print(s,     4);
  Serial.print(",cos:");  Serial.print(c,     4);
  Serial.print(",noise:"); Serial.println(noise, 4);

  t += DT;
}

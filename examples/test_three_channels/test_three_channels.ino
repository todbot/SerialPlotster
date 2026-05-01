/**
 * test_three_channels.ino
 *
 * Outputs three labelled channels at RATE_HZ in SerialPlotster format:
 *   sin:0.9511,cos:0.3090,noise:0.1234
 *
 * The noise signal is a deterministic pseudo-noise (two mixed sinusoids)
 * that matches the SerialPlotster built-in mock stream.
 *
 * Uses micros() so rates above 1000 Hz work correctly.
 * Note: rates above ~2000 Hz require USB CDC (RP2040, Leonardo, etc.) —
 * a hardware UART at 115200 baud tops out around 250 lines/sec.
 */

static const int           RATE_HZ     = 10000;
static const unsigned long INTERVAL_US = 1000000UL / RATE_HZ;

static unsigned long lastUs = 0;

void setup() {
  Serial.begin(115200);
  while (!Serial) { /* wait for USB CDC on Leonardo / 32u4 boards */ }
}

void loop() {
  unsigned long now = micros();
  if (now - lastUs < INTERVAL_US) return;
  lastUs += INTERVAL_US;   // drift-free: advance by fixed step, not now

  // t = actual elapsed seconds so waveform frequency stays correct even when
  // the serial link is slower than RATE_HZ (e.g. USB CDC throughput limit).
  float t     = now * 1e-6f;
  float s     = sinf(t);
  float c     = cosf(t);
  float noise = (sinf(t * 7.31f) * 0.7f + cosf(t * 3.17f) * 0.3f) * 0.5f;

  // Single write per line is much more efficient than multiple Serial.print()
  // calls (avoids repeated USB packet flushes at high data rates).
  char buf[48];
  int  n = snprintf(buf, sizeof(buf), "sin:%.4f,cos:%.4f,noise:%.4f\r\n", s, c, noise);
  Serial.write(buf, n);
}

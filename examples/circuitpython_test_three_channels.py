#
# Outputs three labelled channels at 100 Hz in SerialPlotster format:
#   sin:0.9511,cos:0.3090,noise:0.1234
#
# The noise signal is a deterministic pseudo-noise (two mixed sinusoids)
# that matches the SerialPlotster built-in mock stream.
#
# Compatible with any CircuitPython board at 115200 baud.

import time
from math import sin,cos

# in case there's a display, turn it off, as it slows down serial output
import displayio
displayio.release_displays()

RATE_HZ     = 100;
DT          = 1.0 / RATE_HZ;

t = 0
while True:
    s = sin(t);
    c = cos(t);
    noise = (sin(t * 7.31) * 0.7 + cos(t * 3.17) * 0.3) * 0.5;
    print("sin:%.4f, cos:%.4f, noise:%.4f" % (s,c,noise))
    t += DT;
    time.sleep(DT)

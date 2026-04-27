import displayio
displayio.release_displays()

import time, board, synthio
import ulab.numpy as np

SAMPLE_SIZE = 1024
SAMPLE_VOLUME = 32767
ramp = np.linspace(-SAMPLE_VOLUME, SAMPLE_VOLUME, SAMPLE_SIZE, endpoint=False, dtype=np.int16)
sine = np.array(
    np.sin(np.linspace(0, 2 * np.pi, SAMPLE_SIZE, endpoint=False)) * SAMPLE_VOLUME,
    dtype=np.int16,
)

#import audioio
#audio = audioio.AudioOut(board.A0)  # e.g. ESP32
#import audiopwomio
#audio = audiopwmio.PWMAudioOut(board.GP10)  # e.g. Pico
import audiobusio
audio = audiobusio.I2SOut(bit_clock=board.IO35, word_select=board.IO36, data=board.IO37)  # everyone
synth = synthio.Synthesizer(sample_rate=22050)
audio.play(synth)  # start the synth engine so we can use LFOs

lfo1 = synthio.LFO(ramp, rate=1, offset=1)
lfo2 = synthio.LFO(sine, rate=1.5, offset=0, scale=4)
lfo3 = synthio.LFO(sine, rate=lfo2, offset=-2, scale=lfo1)
lfos = [lfo1, lfo2, lfo3]

synth.blocks[:] = lfos  # attach LFOs to synth so they get ticked

while True:
    #print("(", ",".join(str(lfo.value) for lfo in lfos), ")" )
    print("lfo1:", lfo1.value, "lfo2:", lfo2.value, "lfo3:", lfo3.value)
    time.sleep(0.05)



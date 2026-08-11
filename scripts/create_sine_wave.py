import numpy as np
import sounddevice as sd

# Parameters
frequency = 440 # Frequency in Hz (A4 note)
duration = 0.7 # Duration in seconds
sampling_rate = 44100 # Samples per second
# Generate sine wave samples
t = np.linspace(0, duration, int(sampling_rate * duration), False)
sine_wave = np.sin(2 * np.pi * frequency * t)
# Play the sine wave sound
sd.play(sine_wave, sampling_rate)
sd.wait()

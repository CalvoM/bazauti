import soundfile as sf
import torch
import torchaudio
from silero_vad import get_speech_timestamps, load_silero_vad

TARGET_SAMPLE_RATE = 16000


def read_audio(path: str) -> torch.Tensor:
    data, sample_rate = sf.read(path, dtype="float32", always_2d=True)
    wav = torch.from_numpy(data.T).mean(dim=0)
    if sample_rate != TARGET_SAMPLE_RATE:
        wav = torchaudio.functional.resample(wav, sample_rate, TARGET_SAMPLE_RATE)
    return wav


model = load_silero_vad()
wav = read_audio("audio_files/audio1.mp3")
speech_timestamps = get_speech_timestamps(audio=wav, model=model, return_seconds=True)
print(speech_timestamps)

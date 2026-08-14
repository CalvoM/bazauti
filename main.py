from bazauti.audio_parser import WAVParser


def main():
    wav_parser = WAVParser("audio_files/he-man.wav")
    print(wav_parser.metadata)


if __name__ == "__main__":
    main()

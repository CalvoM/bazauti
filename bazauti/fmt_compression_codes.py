
from enum import StrEnum


class FmtCompressionCode(StrEnum):
    """Ref: https://www.recordingblogs.com/wiki/format-chunk-of-a-wave-file"""
    UNKNOWN = "Unknown"
    PCM = "Microsoft PCM (Uncompressed)"
    ADPCM = "Microsoft ADPCM"
    IEEE_FLOAT = "Microsoft IEEE Float"
    G711_ALAW = "ITU G.711 a-law"
    G711_ULAW = "ITU G.711 u-law"
    IMA_ADPCM = "Intel IMA/DVI ADPCM"
    GSM610 = "Microsoft GSM610"
    DOLBY_AC2 = "Dolby AC2"
    MS_MPEG = "Microsoft MPEG"
    MP3 = "MP3"
    AAC = "AAC"
    FLAC = "Free Lossless Audio Codec FLAC"
    EXTENSIBLE = "Extensible"


def parse_compression_code(code: int) -> FmtCompressionCode:
    match code:
        case 1:
            return FmtCompressionCode.PCM
        case 2:
            return FmtCompressionCode.ADPCM
        case 3:
            return FmtCompressionCode.IEEE_FLOAT
        case 6:
            return FmtCompressionCode.G711_ALAW
        case 7:
            return FmtCompressionCode.G711_ULAW
        case 17:
            return FmtCompressionCode.IMA_ADPCM
        case 49:
            return FmtCompressionCode.GSM610
        case 48:
            return FmtCompressionCode.DOLBY_AC2
        case 80:
            return FmtCompressionCode.MS_MPEG
        case 85:
            return FmtCompressionCode.MP3
        case 255:
            return FmtCompressionCode.AAC
        case 61868:
            return FmtCompressionCode.FLAC
        case 65534:
            return FmtCompressionCode.EXTENSIBLE
        case _:
            return FmtCompressionCode.UNKNOWN

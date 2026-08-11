import logging
from dataclasses import dataclass, field
from typing import Any

from bazauti.exceptions import InvalidFileHeaderError

logger = logging.getLogger(__name__)

@dataclass
class WAVMetadataSubChunk:
    name: str = ""
    size: int = 0
    properties: dict[str, Any] = field(default_factory=dict)

@dataclass
class WAVMetadata:
    sample_rate: float = 0.0
    file_size: int = 0
    sub_chunks: list[WAVMetadataSubChunk] = field(default_factory=list)

SUB_CHUNK_ID_BYTE_SIZE = 4
SUB_CHUNK_SIZE_BYTE_SIZE = 4

class WAVParser:
    def __init__(self, input_file:str):
        self.input_file: str = input_file
        self._metadata = WAVMetadata()
        self._parse_metadata()

    @property
    def metadata(self) -> WAVMetadata:
        """WAV file metadata"""
        return self._metadata

    def _parse_metadata(self):
        try:
            file_data: bytes

            with open(self.input_file, "rb") as wav_file:
                file_data = wav_file.read()

            expected_header = file_data[0:4]
            if expected_header != b'RIFF':
                logger.exception("We could not get the expected header: 'RIFF'")
                raise InvalidFileHeaderError(f"WAV File Header parsing failed, expected 'RIFF': found {expected_header}")

            file_size = int.from_bytes(file_data[4:8], 'little') + 8
            self._metadata.file_size = file_size

            wave_format = file_data[8:12]
            if wave_format != b'WAVE':
                raise InvalidFileHeaderError(f"WAV File Header parsing failed: expected 'WAVE': found {wave_format}")

            start_idx = 12
            while start_idx <= (file_size-1):
                sub_chunk_id: str = file_data[start_idx: start_idx + SUB_CHUNK_ID_BYTE_SIZE].decode()
                start_idx += SUB_CHUNK_ID_BYTE_SIZE
                sub_chunk_size: int = int.from_bytes(file_data[start_idx: start_idx + SUB_CHUNK_SIZE_BYTE_SIZE], "little")
                data: bytes = file_data[start_idx + SUB_CHUNK_SIZE_BYTE_SIZE: start_idx + sub_chunk_size +
                                        SUB_CHUNK_SIZE_BYTE_SIZE]
                properties = {}
                if sub_chunk_id == "bext":
                    properties = self._parse_bext_metadata(data)

                start_idx += sub_chunk_size + SUB_CHUNK_SIZE_BYTE_SIZE
                self._metadata.sub_chunks.append(WAVMetadataSubChunk(name=sub_chunk_id, size=sub_chunk_size,
                                                                     properties=properties))
        except Exception as e:
            logger.error(e)
            raise 

    def _parse_bext_metadata(self, data: bytes) -> dict[str, Any]:
        """Parsing bext metadata to get data.
        Ref: https://wavref.til.cafe/chunk/bext/
        """
        properties = {}
        properties["description"] = data[0:256].strip(b'\x00').decode()
        properties["originator"] = data[256:288].strip(b'\x00').decode()
        properties["originator reference"] = data[288:320].strip(b'\x00').decode()
        properties["originator date"] = data[320:330].strip(b'\x00').decode()
        properties["originator time"] = data[330:338].strip(b'\x00').decode()
        properties["time reference"] = int.from_bytes(data[338:346], "little")
        properties["version"] = int.from_bytes(data[346:348], "little")
        properties["umid"] = data[348:412].strip(b'\x00').hex()
        properties["loudness"] = int.from_bytes(data[412:414],"little", signed=True)
        properties["loudness range"] = int.from_bytes(data[414:416],"little", signed=True)
        properties["maximum true peak"] = int.from_bytes(data[416:418],"little", signed=True)
        properties["maximum momentary loudness"] = int.from_bytes(data[418:420],"little", signed=True)
        properties["maximum short term loudness"] = int.from_bytes(data[420:422],"little", signed=True)
        properties["reserved"] = data[422:602].hex()
        properties["coding history"] = data[602: len(data)].strip(b'\x00').decode()
        return properties




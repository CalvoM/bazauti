import logging
from dataclasses import dataclass, field
from typing import Any

from bazauti.exceptions import InvalidFileHeaderError
from bazauti.fmt_compression_codes import parse_compression_code

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
    description: str = ""
    originator: str = ""
    originator_reference: str = ""
    originator_date: str = ""
    originator_time: str = ""
    version: int = 0
    coding_history: str = ""
    compression_code: str = ""
    number_of_channels: int = 0
    bytes_per_second: int = 0
    block_align: int = 0
    bits_per_sample: int = 0
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
                elif sub_chunk_id == "fmt ":
                    properties = self._parse_fmt_metadata(data)
                elif sub_chunk_id == "data":
                    properties = self._parse_data_metadata(data)
                elif sub_chunk_id == "LIST":
                    self._parse_list_metadata(data)
                elif sub_chunk_id == "fact":
                    properties = self._parse_fact_metadata(data)

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
        if properties["version"] != 1 and properties["version"] != 2:
            properties["reserved"] = data[422:602].hex()
        properties["coding history"] = data[602: len(data)].strip(b'\x00').decode()

        self._metadata.description = properties["description"]
        self._metadata.originator = properties["originator"]
        self._metadata.originator_reference = properties["originator reference"]
        self._metadata.originator_date = properties["originator date"]
        self._metadata.originator_time = properties["originator time"]
        self._metadata.version = properties["version"]
        self._metadata.coding_history = properties["coding history"]

        return properties

    def _parse_fmt_metadata(self, data:bytes) -> dict[str, Any]:
        properties = {}
        compression_code = parse_compression_code(int.from_bytes(data[0:2], "little"))
        properties["compression code"] = str(compression_code)
        properties["number of channels"] = int.from_bytes(data[2:4], "little")
        properties["sample rate"] = int.from_bytes(data[4:8], "little")
        properties["Bytes per second"] = int.from_bytes(data[8:12], "little")
        properties["Block align"] = int.from_bytes(data[12:14], "little")
        properties["bits per sample"] = int.from_bytes(data[14:16], "little")
        no_of_extra_format_bytes = int.from_bytes(data[16:18], "little")
        properties["extra format bytes"] = data[18:18+no_of_extra_format_bytes]

        self._metadata.compression_code = properties["compression code"]
        self._metadata.number_of_channels = properties["number of channels"]
        self._metadata.sample_rate = properties["sample rate"]
        self._metadata.bytes_per_second = properties["Bytes per second"]
        self._metadata.block_align = properties["Block align"]
        self._metadata.bits_per_sample = properties["bits per sample"]

        return properties

    def _parse_data_metadata(self, data: bytes) -> dict[str, Any]:
        properties = {}
        return properties

    def _parse_list_metadata(self, data: bytes) -> dict[str, Any]:
        properties = {}
        list_type_id = data[0:4].strip(b'\x00').decode()
        print(list_type_id)
        start_idx = 4
        while start_idx < (len(data) - 1):
            sub_chunk_id = data[start_idx: start_idx+4].decode()
            print(f"\tID: {sub_chunk_id}")
            start_idx += 4
            sub_chunk_size = int.from_bytes(data[start_idx: start_idx+4], "little")
            start_idx += 4
            sub_chunk_data = data[start_idx: start_idx+sub_chunk_size].strip(b'\x00').decode()
            start_idx += sub_chunk_size
            print(f"\tData: {sub_chunk_data}")
        return properties

    def _parse_fact_metadata(self, data: bytes) -> dict[str, Any]:
        properties = {}
        sample_count = int.from_bytes(data, "little")
        properties["sample_count"] = sample_count
        return properties




use crate::parser::{
    utils::{convert_to_number, fixed_string, CompressionCode, ListInfoId},
    wav::{ima_adpcm::ImaAdpcmCodec, ms_adpcm::MSAdpcmCodec, pcm::PCMCodec},
};
use std::{any::Any, collections::HashMap, fmt::Debug, fs};

use crate::parser::errors::AudioParserError::{self, InvalidFileHeaderError};

#[derive(Clone, Debug)]
pub enum AudioData {
    U8(Vec<u8>),
    I16(Vec<i16>),
}

/// Represents the data in the "bext" sub chunk
#[derive(Debug, Clone)]
pub struct BextMetadata {
    pub description: String,
    pub originator: String,
    pub originator_reference: String,
    pub originator_date: String,
    pub originator_time: String,
    pub time_reference: u64,
    pub version: u16,
    pub umid: [u8; 64],
    pub loudness_value: i16,
    pub loudness_range: i16,
    pub max_true_peak_level: i16,
    pub max_momentary_loudness: i16,
    pub max_short_term_loudness: i16,
    pub reserved: [u8; 180],
    pub coding_history: String,
}

/// Represents the data in the "fmt" sub chunk
#[derive(Debug, Clone, Default)]
pub struct FmtMetadata {
    /// Format Tag of the WAV File e.g. PCM, ADPCM etc.
    pub compression_code: CompressionCode,
    /// Number of channels in the wav file data e.g. 1 for mono and 2 for stereo.
    pub number_of_channels: u16,
    /// Number of samples captured per second.
    pub sample_rate_per_second: u32,
    /// Number of bytes captured per second. Formula changes depending with the compression code.
    pub avg_bytes_per_second: u32,
    /// Size in bytes of a single sample frame.
    pub block_align: u16,
    /// Number of bits in a single sample frame.
    pub bits_per_sample: u16,
    /// WAVEFORMATEX(NSIBLE) ONLY: the number of extra bytes coming after.
    pub extra_bytes_size: Option<u16>,
    /// WAVEFORMATEX(NSIBLE) ONLY: Extra bytes to process, len == [`Self::extra_bytes_size`]
    pub extra_bytes: Option<Vec<u8>>,
    /// Number of sample per ADPCM block
    pub samples_per_block: Option<u16>,
    /// Number of coefficients vars
    pub coefficient_count: Option<u16>,
    /// Coefficients used to (en/de)code the data in WAVEFORMATEX(NSIBLE)
    pub coefficients: Option<Vec<u8>>, //TODO: We need more info.
}

/// Represents the metadata for the WAV file.
#[derive(Clone, Debug, Default)]
pub struct WAVMetadata {
    pub file_size: u32,
    pub bext_metadata: Option<BextMetadata>,
    pub fmt_metadata: FmtMetadata,
    pub data_metadata: Vec<u8>,
}
#[derive(Debug)]
pub struct WAVParser {
    input_file: String,
    raw_metadata: WAVMetadata,
    raw_data: Option<AudioData>,
    codec: Option<Box<dyn WAVCodec>>,
}

#[derive(Debug)]
pub struct WaveParserContext {
    pub samples: Option<AudioData>,
    pub sample_rate: u32,
    pub channels: u16,
    pub compression_code: CompressionCode,
}

pub trait WAVCodec: Debug {
    fn compression_code(&self) -> CompressionCode;
    fn parse_audio_data(
        &self,
        data: &[u8],
        metadata: &FmtMetadata,
    ) -> Result<AudioData, AudioParserError>;
    fn parse_fmt_metadata(&self, data: &[u8]) -> Result<FmtMetadata, AudioParserError>;
    fn parse_base_fmt_metadata(&self, data: &[u8]) -> Result<FmtMetadata, AudioParserError> {
        let number_of_channels = convert_to_number::<u16>(data, 2, 4).unwrap();
        if number_of_channels < 1 {
            return Err(InvalidFileHeaderError(format!(
                "The channels should be more than 1, instead we got: {}",
                number_of_channels
            )));
        }
        let sample_rate_per_second = convert_to_number::<u32>(data, 4, 8).unwrap();
        let avg_bytes_per_second = convert_to_number::<u32>(data, 8, 12).unwrap();
        let block_align = convert_to_number::<u16>(data, 12, 14).unwrap();
        let bits_per_sample = convert_to_number::<u16>(data, 14, 16).unwrap();
        let extra_bytes_size: Option<u16> = None;
        let extra_bytes: Option<Vec<u8>> = None;
        let samples_per_block: Option<u16> = None;
        let coefficient_count: Option<u16> = None;
        let coefficients: Option<Vec<u8>> = None;
        let metadata = FmtMetadata {
            compression_code: self.compression_code(),
            number_of_channels,
            sample_rate_per_second,
            avg_bytes_per_second,
            block_align,
            bits_per_sample,
            extra_bytes_size,
            extra_bytes,
            samples_per_block,
            coefficient_count,
            coefficients,
        };
        Ok(metadata)
    }
}

impl WAVParser {
    pub fn new(file_name: &str) -> Self {
        Self {
            input_file: String::from(file_name),
            raw_metadata: WAVMetadata::default(),
            raw_data: None,
            codec: None,
        }
    }

    pub fn metadata(&self) -> WAVMetadata {
        self.raw_metadata.clone()
    }

    pub fn samples(self) -> Vec<u8> {
        self.raw_metadata.data_metadata
    }

    pub fn context(&self) -> WaveParserContext {
        WaveParserContext {
            samples: self.raw_data.clone(),
            sample_rate: self.raw_metadata.fmt_metadata.sample_rate_per_second,
            channels: self.raw_metadata.fmt_metadata.number_of_channels,
            compression_code: self.raw_metadata.fmt_metadata.compression_code,
        }
    }

    pub fn parse(&mut self) -> Result<(), AudioParserError> {
        let file_data = fs::read(&self.input_file)
            .map_err(|e| AudioParserError::InputAudioFileError(e.to_string()))?;

        // Check the header
        let expected_header = &file_data[0..4];
        if expected_header != "RIFF".as_bytes() {
            return Err(InvalidFileHeaderError(format!(
                "WAV File Header parsing failed, expected 'RIFF': found {}",
                1
            )));
        }

        // Get the file size
        let file_size: u32 = u32::from_le_bytes(file_data[4..8].try_into().unwrap());
        self.raw_metadata.file_size = file_size;

        // Check wave format
        let wave_format = &file_data[8..12];
        if wave_format != "WAVE".as_bytes() {
            return Err(InvalidFileHeaderError(format!(
                "WAV file Header parsing failed: expected 'WAVE': found {}",
                1
            )));
        }

        let mut start_idx: usize = 12;
        while start_idx <= (file_size - 1).try_into().unwrap() {
            let sub_chunk_id = std::str::from_utf8(&file_data[start_idx..start_idx + 4])
                .unwrap()
                .to_string();
            start_idx += 4;
            let mut sub_chunk_size: usize =
                u32::from_le_bytes(file_data[start_idx..start_idx + 4].try_into().unwrap())
                    .try_into()
                    .unwrap();
            if sub_chunk_size % 2 != 0 {
                // Data should be word aligned, thus must be even.
                sub_chunk_size += 1;
            }
            let data: &[u8] = &file_data[start_idx + 4..start_idx + sub_chunk_size + 4];
            if sub_chunk_id == String::from("bext") {
                let metadata = self.parse_bext_metadata(data).unwrap();
                self.raw_metadata.bext_metadata = Some(metadata);
            } else if sub_chunk_id == String::from("fmt ") {
                let metadata = self.parse_fmt_metadata(data).unwrap();
                self.raw_metadata.fmt_metadata = metadata;
            } else if sub_chunk_id == String::from("data") {
                self.raw_metadata.data_metadata = data.to_vec();
            } else if sub_chunk_id == String::from("LIST") {
                self.parse_list_metadata(data);
            } else if sub_chunk_id == String::from("fact") {
                self.parse_fact_metadata(data);
            } else if sub_chunk_id == String::from("cue ") {
                self.parse_cue_metadata(data);
            }
            start_idx += sub_chunk_size + 4;
        }
        self.parse_audio_data();
        Ok(())
    }
    fn parse_bext_metadata(&mut self, data: &[u8]) -> Result<BextMetadata, AudioParserError> {
        let metadata = BextMetadata {
            description: fixed_string(&data[0..256]),
            originator: fixed_string(&data[256..288]),
            originator_reference: fixed_string(&data[288..320]),
            originator_date: fixed_string(&data[320..330]),
            originator_time: fixed_string(&data[330..338]),
            time_reference: convert_to_number(data, 338, 346).unwrap(),
            version: convert_to_number(data, 346, 348).unwrap(),
            umid: data[348..412].try_into().expect("slice length checked"),
            loudness_value: convert_to_number(data, 412, 414).unwrap(),
            loudness_range: convert_to_number(data, 414, 416).unwrap(),
            max_true_peak_level: convert_to_number(data, 416, 418).unwrap(),
            max_momentary_loudness: convert_to_number(data, 418, 420).unwrap(),
            max_short_term_loudness: convert_to_number(data, 420, 422).unwrap(),
            reserved: data[422..602].try_into().expect("slice length checked"),
            coding_history: fixed_string(&data[602..]),
        };
        Ok(metadata)
    }
    fn parse_fmt_metadata(&mut self, data: &[u8]) -> Result<FmtMetadata, AudioParserError> {
        let compression_code = CompressionCode::from(convert_to_number::<u16>(data, 0, 2).unwrap());

        let codec: Box<dyn WAVCodec> = match compression_code {
            CompressionCode::Pcm => Box::new(PCMCodec {}),
            CompressionCode::MSAdpcm => Box::new(MSAdpcmCodec {}),
            CompressionCode::ImaAdpcm => Box::new(ImaAdpcmCodec {}),
            _ => todo!(),
        };
        let metadata = codec.parse_fmt_metadata(data);
        self.codec = Some(codec);
        metadata
    }
    fn parse_list_metadata(&mut self, data: &[u8]) -> HashMap<String, String> {
        let mut properties: HashMap<String, String> = HashMap::new();
        let list_type_id = fixed_string(&data[0..4]);
        if list_type_id == String::from("INFO") {
            properties = self.parse_list_info_metadata(&data[4..]);
        } else if list_type_id == String::from("adtl") {
            properties = self.parse_list_adtl_metadata(&data[4..]);
        } else if list_type_id == String::from("wavl") {
            properties = self.parse_list_wavl_metadata(&data[4..]);
        }
        properties
    }
    fn parse_list_info_metadata(&mut self, data: &[u8]) -> HashMap<String, String> {
        let mut properties: HashMap<String, String> = HashMap::new();
        let mut idx = 0;
        while idx < (data.len() - 1) {
            let sub_chunk_id: [u8; 4] = data[idx..idx + 4]
                .try_into()
                .expect("sub chunk id should have a length of 4");
            idx += 4;
            let mut sub_chunk_size = convert_to_number::<u32>(data, idx, idx + 4).unwrap();
            idx += 4;
            if sub_chunk_size % 2 != 0 {
                sub_chunk_size += 1;
            }
            let sub_chunk_data = fixed_string(&data[idx..(idx + sub_chunk_size as usize)]);
            idx += sub_chunk_size as usize;
            let info_id = ListInfoId::from(sub_chunk_id);
            properties.insert(format!("{info_id}"), sub_chunk_data);
        }
        properties
    }
    fn parse_list_adtl_metadata(&mut self, _data: &[u8]) -> HashMap<String, String> {
        let properties: HashMap<String, String> = HashMap::new();
        properties
    }
    fn parse_list_wavl_metadata(&mut self, _data: &[u8]) -> HashMap<String, String> {
        let properties: HashMap<String, String> = HashMap::new();
        properties
    }
    fn parse_fact_metadata(&mut self, data: &[u8]) -> HashMap<String, Box<dyn Any>> {
        let mut properties: HashMap<String, Box<dyn Any>> = HashMap::new();
        let sample_count = u32::from_le_bytes(data.try_into().unwrap());
        properties.insert(String::from("sample_count"), Box::new(sample_count));
        properties
    }
    fn parse_cue_metadata(&mut self, _data: &[u8]) -> HashMap<String, Box<dyn Any>> {
        let properties: HashMap<String, Box<dyn Any>> = HashMap::new();
        properties
    }
    fn parse_audio_data(&mut self) {
        let Some(codec) = &self.codec else {
            return;
        };
        if let Ok(audio_data) = codec.parse_audio_data(
            &self.raw_metadata.data_metadata,
            &self.raw_metadata.fmt_metadata,
        ) {
            self.raw_data = Some(audio_data);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}

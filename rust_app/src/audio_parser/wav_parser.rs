use crate::audio_parser::utils::{
    convert_to_number, fixed_string, parse_compression_code, CompressionCode,
};
use std::{any::Any, collections::HashMap, convert, fs};

use crate::audio_parser::errors::AudioParserError::{self, InvalidFileHeaderError};

#[derive(Debug, Default)]
pub struct WAVMetadataSubChunk {
    pub name: String,
    pub size: u32,
}

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

#[derive(Debug, Clone, Default)]
pub struct FmtMetadata {
    pub compression_code: CompressionCode,
    pub number_of_channels: u16,
    pub sample_rate_per_second: u32,
    pub avg_bytes_per_second: u32,
    pub block_align: u16,
    pub bits_per_sample: u16,
    pub extra_bytes_size: Option<u16>,
    pub extra_bytes: Option<Vec<u8>>,
    pub samples_per_block: Option<u16>,
    pub coefficient_count: Option<u16>,
    pub coefficients: Option<Vec<u8>>, //TODO: We need more info.
}

#[derive(Debug, Default)]
pub struct WAVMetadata {
    //pub sample_rate: u32,
    pub file_size: u32,
    //pub description: String,
    //pub originator: String,
    //pub originator_reference: String,
    //pub originator_date: String,
    //pub originator_time: String,
    //pub version: u16,
    //pub coding_history: String,
    //pub compression_code: String,
    //pub number_of_channels: u16,
    //pub bytes_per_second: u16,
    //pub block_align: u16,
    //pub bits_per_sample: u32,
    pub bext_metadata: Option<BextMetadata>,
    pub fmt_metadata: Option<FmtMetadata>,
}
pub struct WAVParser {
    input_file: String,
    raw_metadata: WAVMetadata,
    raw_data: Vec<u8>,
}

impl WAVParser {
    pub fn new(file_name: &str) -> Self {
        Self {
            input_file: String::from(file_name),
            raw_metadata: WAVMetadata::default(),
            raw_data: Vec::new(),
        }
    }

    pub fn metadata(self) -> WAVMetadata {
        self.raw_metadata
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
            let sub_chunk_size: usize =
                u32::from_le_bytes(file_data[start_idx..start_idx + 4].try_into().unwrap())
                    .try_into()
                    .unwrap();
            let data: &[u8] = &file_data[start_idx + 4..start_idx + sub_chunk_size + 4];
            if sub_chunk_id == String::from("bext") {
                let metadata = self.parse_bext_metadata(data).unwrap();
                self.raw_metadata.bext_metadata = Some(metadata);
            } else if sub_chunk_id == String::from("fmt ") {
                let metadata = self.parse_fmt_metadata(data).unwrap();
                self.raw_metadata.fmt_metadata = Some(metadata);
            } else if sub_chunk_id == String::from("data") {
                let data = self.parse_data_metadata(data);
                self.raw_data = data.unwrap();
            } else if sub_chunk_id == String::from("LIST") {
                self.parse_list_metadata(data);
            } else if sub_chunk_id == String::from("fact") {
                self.parse_fact_metadata(data);
            } else if sub_chunk_id == String::from("cue ") {
                self.parse_cue_metadata(data);
            }
            start_idx += sub_chunk_size + 4;
        }
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
        let compression_code = parse_compression_code(convert_to_number(data, 0, 2).unwrap());

        let mut extra_bytes_size: Option<u16> = None;
        let mut extra_bytes: Option<Vec<u8>> = None;
        let mut samples_per_block: Option<u16> = None;
        let mut coefficient_count: Option<u16> = None;
        let mut coefficients: Option<Vec<u8>> = None;

        if compression_code == CompressionCode::Pcm {
            extra_bytes_size = None;
            extra_bytes = None;
            samples_per_block = None;
            coefficient_count = None;
            coefficients = None;
        } else if compression_code == CompressionCode::Adpcm {
            extra_bytes_size = Some(convert_to_number(data, 16, 18).unwrap());
            samples_per_block = Some(convert_to_number(data, 18, 20).unwrap());
            coefficient_count = Some(convert_to_number(data, 20, 22).unwrap());
            if coefficient_count.unwrap() > 0 {
                let limit = 8 * coefficient_count.unwrap() as usize;
                coefficients = Some(data[24..limit].to_vec());
            }
        } else if compression_code == CompressionCode::ImaAdpcm {
            extra_bytes_size = Some(convert_to_number(data, 16, 18).unwrap());
            samples_per_block = Some(convert_to_number(data, 18, 20).unwrap());
        } else {
            extra_bytes_size = Some(convert_to_number(data, 16, 18).unwrap());
            if extra_bytes_size.unwrap() > 0 {
                let limit = extra_bytes_size.unwrap() as usize;
                extra_bytes = Some(data[18..limit].to_vec());
            }
        }
        let metadata = FmtMetadata {
            compression_code,
            number_of_channels: convert_to_number(data, 2, 4).unwrap(),
            sample_rate_per_second: convert_to_number(data, 4, 8).unwrap(),
            avg_bytes_per_second: convert_to_number(data, 8, 12).unwrap(),
            block_align: convert_to_number(data, 12, 14).unwrap(),
            bits_per_sample: convert_to_number(data, 14, 16).unwrap(),
            extra_bytes_size,
            extra_bytes,
            samples_per_block,
            coefficient_count,
            coefficients,
        };
        Ok(metadata)
    }
    fn parse_data_metadata(&mut self, data: &[u8]) -> Result<Vec<u8>, AudioParserError> {
        Ok(data.to_vec())
    }
    fn parse_list_metadata(&mut self, data: &[u8]) -> HashMap<String, Box<dyn Any>> {
        let mut properties: HashMap<String, Box<dyn Any>> = HashMap::new();
        properties
    }
    fn parse_list_info_metadata(&mut self, data: &[u8]) -> HashMap<String, Box<dyn Any>> {
        let mut properties: HashMap<String, Box<dyn Any>> = HashMap::new();
        properties
    }
    fn parse_list_adtl_metadata(&mut self, data: &[u8]) -> HashMap<String, Box<dyn Any>> {
        let mut properties: HashMap<String, Box<dyn Any>> = HashMap::new();
        properties
    }
    fn parse_list_wavl_metadata(&mut self, data: &[u8]) -> HashMap<String, Box<dyn Any>> {
        let mut properties: HashMap<String, Box<dyn Any>> = HashMap::new();
        properties
    }
    fn parse_fact_metadata(&mut self, data: &[u8]) -> HashMap<String, Box<dyn Any>> {
        let mut properties: HashMap<String, Box<dyn Any>> = HashMap::new();
        let sample_count = u32::from_le_bytes(data.try_into().unwrap());
        properties.insert(String::from("sample_count"), Box::new(sample_count));
        properties
    }
    fn parse_cue_metadata(&mut self, data: &[u8]) -> HashMap<String, Box<dyn Any>> {
        let mut properties: HashMap<String, Box<dyn Any>> = HashMap::new();
        properties
    }
}

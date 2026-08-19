use std::{any::Any, collections::HashMap, fs};

use crate::audio_parser::errors::AudioParserError::{self, InvalidFileHeaderError};

#[derive(Debug, Default)]
pub struct WAVMetadataSubChunk {
    pub name: String,
    pub size: u32,
    pub properties: HashMap<String, Box<dyn Any>>,
}

#[derive(Debug, Default)]
pub struct WAVMetadata {
    pub sample_rate: u32,
    pub file_size: u32,
    pub description: String,
    pub originator: String,
    pub originator_reference: String,
    pub originator_date: String,
    pub originator_time: String,
    pub version: u16,
    pub coding_history: String,
    pub compression_code: String,
    pub number_of_channels: u16,
    pub bytes_per_second: u16,
    pub block_align: u16,
    pub bits_per_sample: u32,
    pub sub_chunks: Vec<WAVMetadataSubChunk>,
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
            let mut properties: HashMap<String, Box<dyn Any>> = HashMap::new();
            if sub_chunk_id == String::from("bext") {
                properties = self.parse_bext_metadata(data);
            } else if sub_chunk_id == String::from("fmt ") {
                properties = self.parse_fmt_metadata(data);
            } else if sub_chunk_id == String::from("data") {
                properties = self.parse_data_metadata(data);
            } else if sub_chunk_id == String::from("LIST") {
                properties = self.parse_list_metadata(data);
            } else if sub_chunk_id == String::from("fact") {
                properties = self.parse_fact_metadata(data);
            } else if sub_chunk_id == String::from("cue ") {
                properties = self.parse_cue_metadata(data);
            }
            start_idx += sub_chunk_size + 4;
            self.raw_metadata.sub_chunks.push(WAVMetadataSubChunk {
                name: sub_chunk_id,
                size: sub_chunk_size.try_into().unwrap(),
                properties,
            });
        }
        Ok(())
    }
    fn parse_bext_metadata(&mut self, data: &[u8]) -> HashMap<String, Box<dyn Any>> {
        let mut properties: HashMap<String, Box<dyn Any>> = HashMap::new();
        properties.insert(
            String::from("description"),
            Box::new(
                String::from_utf8_lossy(&data[0..256].split(|&x| x == 0).next().unwrap_or(&[]))
                    .into_owned(),
            ),
        );
        properties.insert(
            String::from("originator"),
            Box::new(
                String::from_utf8_lossy(&data[256..288].split(|&x| x == 0).next().unwrap_or(&[]))
                    .into_owned(),
            ),
        );
        properties.insert(
            String::from("originator reference"),
            Box::new(
                String::from_utf8_lossy(&data[288..320].split(|&x| x == 0).next().unwrap_or(&[]))
                    .into_owned(),
            ),
        );
        properties.insert(
            String::from("originator date"),
            Box::new(
                String::from_utf8_lossy(&data[320..330].split(|&x| x == 0).next().unwrap_or(&[]))
                    .into_owned(),
            ),
        );
        properties.insert(
            String::from("originator time"),
            Box::new(
                String::from_utf8_lossy(&data[330..338].split(|&x| x == 0).next().unwrap_or(&[]))
                    .into_owned(),
            ),
        );
        properties.insert(
            String::from("time reference"),
            Box::new(u64::from_le_bytes(data[338..346].try_into().unwrap())),
        );
        let version = u16::from_le_bytes(data[346..348].try_into().unwrap());
        properties.insert(String::from("version"), Box::new(version));
        properties.insert(
            String::from("umid"),
            Box::new(
                data[348..412]
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>(),
            ),
        );
        properties.insert(
            String::from("loudness"),
            Box::new(u16::from_le_bytes(data[412..414].try_into().unwrap())),
        );
        properties.insert(
            String::from("loudness range"),
            Box::new(u16::from_le_bytes(data[414..416].try_into().unwrap())),
        );
        properties.insert(
            String::from("maximum true peak"),
            Box::new(u16::from_le_bytes(data[416..418].try_into().unwrap())),
        );
        properties.insert(
            String::from("maximum momentary loudness"),
            Box::new(u16::from_le_bytes(data[418..420].try_into().unwrap())),
        );
        properties.insert(
            String::from("maximum short term loudness"),
            Box::new(u16::from_le_bytes(data[420..422].try_into().unwrap())),
        );
        if version != 1 && version != 2 {
            properties.insert(
                String::from("reserved"),
                Box::new(
                    data[422..602]
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<String>(),
                ),
            );
        }
        let coding_history = String::from_utf8_lossy(
            &data[602..data.len()]
                .split(|&x| x == 0)
                .next()
                .unwrap_or(&[]),
        );
        properties.insert(
            String::from("coding history"),
            Box::new(coding_history.into_owned()),
        );
        self.raw_metadata.description = properties
            .get("description")
            .unwrap()
            .downcast_ref::<String>()
            .unwrap()
            .clone();
        self.raw_metadata.originator = properties
            .get("originator")
            .unwrap()
            .downcast_ref::<String>()
            .unwrap()
            .clone();
        self.raw_metadata.originator_reference = properties
            .get("originator reference")
            .unwrap()
            .downcast_ref::<String>()
            .unwrap()
            .clone();
        self.raw_metadata.originator_date = properties
            .get("originator date")
            .unwrap()
            .downcast_ref::<String>()
            .unwrap()
            .clone();
        self.raw_metadata.originator_time = properties
            .get("originator time")
            .unwrap()
            .downcast_ref::<String>()
            .unwrap()
            .clone();
        self.raw_metadata.version = properties
            .get("version")
            .unwrap()
            .downcast_ref::<u16>()
            .unwrap()
            .clone();
        self.raw_metadata.coding_history = properties
            .get("coding history")
            .unwrap()
            .downcast_ref::<String>()
            .unwrap()
            .clone();
        properties
    }
    fn parse_fmt_metadata(&mut self, data: &[u8]) -> HashMap<String, Box<dyn Any>> {
        let mut properties: HashMap<String, Box<dyn Any>> = HashMap::new();
        properties
    }
    fn parse_data_metadata(&mut self, data: &[u8]) -> HashMap<String, Box<dyn Any>> {
        let mut properties: HashMap<String, Box<dyn Any>> = HashMap::new();
        self.raw_data = data.to_vec();
        properties.insert(String::from("raw_data"), Box::new(data.to_vec()));
        properties
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

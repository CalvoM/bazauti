use super::base::AudioData;
use crate::parser::errors::AudioParserError;
use crate::parser::utils::CompressionCode;
use crate::parser::{errors::AudioParserError::InvalidFileHeaderError, wav::base::WAVCodec};

#[derive(Debug)]
pub struct PCMCodec {}
impl WAVCodec for PCMCodec {
    fn compression_code(&self) -> CompressionCode {
        CompressionCode::Pcm
    }
    fn parse_audio_data(
        &self,
        data: &[u8],
        fmt_metadata: &super::base::FmtMetadata,
    ) -> Result<AudioData, AudioParserError> {
        let number_of_channels = fmt_metadata.number_of_channels;
        let bits_per_sample = fmt_metadata.bits_per_sample;
        let block_align = fmt_metadata.block_align;
        let mut raw_data = if bits_per_sample == 8 {
            AudioData::U8(Vec::new())
        } else {
            AudioData::I16(Vec::new())
        };
        for sample in data.chunks_exact(block_align as usize) {
            match (&mut raw_data, number_of_channels, bits_per_sample) {
                (AudioData::U8(data), 1, 8) => data.push(sample[0]),
                (AudioData::U8(data), 2, 8) => {
                    let left = sample[0];
                    let right = sample[1];
                    data.push(left);
                    data.push(right);
                }
                (AudioData::I16(data), 1, 16) => {
                    let sample = i16::from_le_bytes([sample[0], sample[1]]);
                    data.push(sample);
                }
                (AudioData::I16(data), 2, 16) => {
                    let left = i16::from_le_bytes([sample[0], sample[1]]);
                    let right = i16::from_le_bytes([sample[2], sample[3]]);

                    data.push(left);
                    data.push(right);
                }
                _ => unreachable!(),
            }
        }
        Ok(raw_data)
    }
    fn parse_fmt_metadata(
        &self,
        data: &[u8],
    ) -> Result<super::base::FmtMetadata, crate::parser::errors::AudioParserError> {
        let mut metadata = self.parse_base_fmt_metadata(data).unwrap();
        let calculated_block_align = metadata.number_of_channels * metadata.bits_per_sample / 8;
        if calculated_block_align != metadata.block_align {
            return Err(InvalidFileHeaderError(format!(
                "Mismatch of the block align: Got: {}, expected: {}",
                calculated_block_align, metadata.block_align
            )));
        }
        let calculated_avg_bytes_per_second =
            metadata.sample_rate_per_second * metadata.block_align as u32;
        if calculated_avg_bytes_per_second != metadata.avg_bytes_per_second {
            return Err(InvalidFileHeaderError(format!(
                "Mismatch of average bytes per second: Got: {}, expected: {}",
                calculated_avg_bytes_per_second, metadata.avg_bytes_per_second
            )));
        }
        metadata.extra_bytes_size = None;
        metadata.extra_bytes = None;
        metadata.samples_per_block = None;
        metadata.coefficient_count = None;
        metadata.coefficients = None;
        Ok(metadata)
    }
}

use crate::parser::errors::AudioParserError::InvalidFileHeaderError;
use crate::parser::utils::{convert_to_number, CompressionCode};
use crate::parser::wav::base::WAVCodec;
#[derive(Debug)]
pub struct ImaAdpcmCodec {}
impl ImaAdpcmCodec {
    fn calculate_dvi_ima_adpcm_samples_per_block(
        block_align: u16,
        number_of_channels: u16,
        bits_per_sample: u16,
    ) -> u16 {
        (((block_align - (4 * number_of_channels)) * 8) / (bits_per_sample * number_of_channels))
            + 1
    }
    fn calculate_adpcm_avg_bytes_per_second(
        sample_rate_per_second: u32,
        samples_per_block: u16,
        block_align: u16,
    ) -> f32 {
        (sample_rate_per_second as f32 / samples_per_block as f32) * block_align as f32
    }
}
impl WAVCodec for ImaAdpcmCodec {
    fn compression_code(&self) -> CompressionCode {
        CompressionCode::ImaAdpcm
    }
    fn parse_audio_data(
        &self,
        _data: &[u8],
        _metadata: &super::base::FmtMetadata,
    ) -> Result<super::base::AudioData, crate::parser::errors::AudioParserError> {
        Ok(super::base::AudioData::U8(vec![]))
    }
    fn parse_fmt_metadata(
        &self,
        data: &[u8],
    ) -> Result<super::base::FmtMetadata, crate::parser::errors::AudioParserError> {
        let mut metadata = self.parse_base_fmt_metadata(data).unwrap();
        metadata.extra_bytes_size = Some(convert_to_number(data, 16, 18).unwrap());
        metadata.samples_per_block = Some(convert_to_number(data, 18, 20).unwrap());
        let calculated_avg_bytes_per_second = Self::calculate_adpcm_avg_bytes_per_second(
            metadata.sample_rate_per_second,
            metadata.samples_per_block.unwrap_or(0),
            metadata.block_align,
        );
        if calculated_avg_bytes_per_second.floor() != metadata.avg_bytes_per_second as f32 {
            return Err(InvalidFileHeaderError(format!(
                "Mismatch of the average bytes per second: Got: {}, expected: {}",
                calculated_avg_bytes_per_second, metadata.avg_bytes_per_second
            )));
        }
        let calculated_samples_per_block = Self::calculate_dvi_ima_adpcm_samples_per_block(
            metadata.block_align,
            metadata.number_of_channels,
            metadata.bits_per_sample,
        );
        if metadata.samples_per_block.unwrap_or(0) != calculated_samples_per_block {
            return Err(InvalidFileHeaderError(format!(
                "Mismatch of the samples per block align: Got: {}, expected: {}",
                calculated_samples_per_block,
                metadata.samples_per_block.unwrap_or(0)
            )));
        } else {
            let calculated_block_align = metadata.number_of_channels * metadata.bits_per_sample / 8;
            if calculated_block_align != metadata.block_align {
                return Err(InvalidFileHeaderError(format!(
                    "Mismatch of the block align: Got: {}, expected: {}",
                    calculated_block_align, metadata.block_align
                )));
            }
            metadata.extra_bytes_size = Some(convert_to_number(data, 16, 18).unwrap());
            if metadata.extra_bytes_size.unwrap() > 0 {
                let limit = metadata.extra_bytes_size.unwrap() as usize;
                metadata.extra_bytes = Some(data[18..limit].to_vec());
            }
        }
        Ok(metadata)
    }
}

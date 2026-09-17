use crate::parser::errors::AudioParserError::InvalidFileHeaderError;
use crate::parser::utils::{convert_to_number, CompressionCode};
use crate::parser::wav::base::{AudioData, WAVCodec};

const ADPCM_BITS_PER_SAMPLE: u16 = 4;

/// https://wiki.multimedia.cx/index.php/Microsoft_ADPCM
const ADPCM_FIXED_COEF_BASE: i32 = 256;
const ADPCM_FIXED_ADAPTATION_BASE: i32 = 256;
const ADAPTATION_TABLE: [i32; 16] = [
    230, 230, 230, 230, 307, 409, 512, 614, 768, 614, 512, 409, 307, 230, 230, 230,
];

#[derive(Debug, Default, Copy, Clone)]
struct ADPCMChannelMetaData {
    predictor: u8,
    sample1: i16,
    sample2: i16,
    coeff1: i16,
    coeff2: i16,
    delta: i16,
}
#[derive(Debug)]
struct ADPCMDecodeContext {
    channels: [ADPCMChannelMetaData; 2],
}

#[derive(Debug)]
pub struct MSAdpcmCodec {}
impl MSAdpcmCodec {
    fn calculate_adpcm_block_align(sample_rate_per_second: u32, number_of_channels: u16) -> u16 {
        let calculated_rate = sample_rate_per_second * number_of_channels as u32;
        let calculated_block_align = if calculated_rate < 22000 {
            256
        } else if calculated_rate > 22000 && calculated_rate < 44000 {
            512
        } else {
            1024
        };
        calculated_block_align
    }
    fn calculate_adpcm_samples_per_block(
        block_align: u16,
        number_of_channels: u16,
        bits_per_sample: u16,
    ) -> u16 {
        (((block_align - (7 * number_of_channels)) * 8) / (bits_per_sample * number_of_channels))
            + 2
    }
    fn calculate_adpcm_avg_bytes_per_second(
        sample_rate_per_second: u32,
        samples_per_block: u16,
        block_align: u16,
    ) -> f32 {
        (sample_rate_per_second as f32 / samples_per_block as f32) * block_align as f32
    }
}
impl WAVCodec for MSAdpcmCodec {
    fn compression_code(&self) -> CompressionCode {
        CompressionCode::MSAdpcm
    }
    fn parse_audio_data(
        &self,
        data: &[u8],
        fmt_metadata: &super::base::FmtMetadata,
    ) -> Result<AudioData, crate::parser::errors::AudioParserError> {
        let coefficients = fmt_metadata.coefficients.as_ref().unwrap();
        let block_align = fmt_metadata.block_align;
        let mut raw_data = AudioData::I16(Vec::new());
        let mut starting_idx = 0;
        let is_stereo = fmt_metadata.number_of_channels == 2;
        let mut decode_ctx = ADPCMDecodeContext {
            channels: [ADPCMChannelMetaData::default(); 2],
        };
        while (starting_idx + block_align as usize) <= data.len() {
            let mut inner_block_idx = 0;
            // Get predictors
            decode_ctx.channels[0].predictor = *data.get(starting_idx + inner_block_idx).unwrap();
            inner_block_idx += 1;
            if is_stereo {
                decode_ctx.channels[1].predictor =
                    *data.get(starting_idx + inner_block_idx).unwrap();
                inner_block_idx += 1;
            }
            // Get Deltas
            decode_ctx.channels[0].delta = convert_to_number::<i16>(
                data,
                starting_idx + inner_block_idx,
                starting_idx + inner_block_idx + 2,
            )
            .unwrap();
            inner_block_idx += 2;
            if is_stereo {
                decode_ctx.channels[1].delta = convert_to_number::<i16>(
                    data,
                    starting_idx + inner_block_idx,
                    starting_idx + inner_block_idx + 2,
                )
                .unwrap();
                inner_block_idx += 2;
            }
            //Get sample1
            decode_ctx.channels[0].sample1 = convert_to_number::<i16>(
                data,
                starting_idx + inner_block_idx,
                starting_idx + inner_block_idx + 2,
            )
            .unwrap();
            inner_block_idx += 2;
            if is_stereo {
                decode_ctx.channels[1].sample1 = convert_to_number::<i16>(
                    data,
                    starting_idx + inner_block_idx,
                    starting_idx + inner_block_idx + 2,
                )
                .unwrap();
                inner_block_idx += 2;
            }
            // Get Sample2
            decode_ctx.channels[0].sample2 = convert_to_number::<i16>(
                data,
                starting_idx + inner_block_idx,
                starting_idx + inner_block_idx + 2,
            )
            .unwrap();
            inner_block_idx += 2;
            if is_stereo {
                decode_ctx.channels[1].sample2 = convert_to_number::<i16>(
                    data,
                    starting_idx + inner_block_idx,
                    starting_idx + inner_block_idx + 2,
                )
                .unwrap();
                inner_block_idx += 2;
            }
            // Calculated the coeff1 and coeff2
            let coefficient_offset = (decode_ctx.channels[0].predictor * 4) as usize;
            decode_ctx.channels[0].coeff1 = convert_to_number::<i16>(
                coefficients,
                coefficient_offset,
                (coefficient_offset) + 2,
            )
            .unwrap();
            decode_ctx.channels[0].coeff2 = convert_to_number::<i16>(
                coefficients,
                coefficient_offset + 2,
                (coefficient_offset) + 4,
            )
            .unwrap();
            if is_stereo {
                let coefficient_offset_ch1 = (decode_ctx.channels[1].predictor * 4) as usize;
                decode_ctx.channels[1].coeff1 = convert_to_number::<i16>(
                    coefficients,
                    coefficient_offset_ch1,
                    (coefficient_offset_ch1) + 2,
                )
                .unwrap();
                decode_ctx.channels[1].coeff2 = convert_to_number::<i16>(
                    coefficients,
                    coefficient_offset_ch1 + 2,
                    (coefficient_offset_ch1) + 4,
                )
                .unwrap();
            }
            if let AudioData::I16(ref mut samples) = raw_data {
                samples.push(decode_ctx.channels[0].sample2);
                if is_stereo {
                    samples.push(decode_ctx.channels[1].sample2);
                }
                samples.push(decode_ctx.channels[0].sample1);
                if is_stereo {
                    samples.push(decode_ctx.channels[1].sample1);
                }
            }
            while inner_block_idx < (block_align as usize) {
                for step in (0..=1).rev() {
                    let ctx_idx: usize = if is_stereo && step == 0 { 1 } else { 0 };
                    let predicted_sample: i32 = ((decode_ctx.channels[ctx_idx].sample1 as i32
                        * decode_ctx.channels[ctx_idx].coeff1 as i32)
                        + (decode_ctx.channels[ctx_idx].sample2 as i32
                            * decode_ctx.channels[ctx_idx].coeff2 as i32))
                        / ADPCM_FIXED_COEF_BASE;
                    let nibble = (*data.get(starting_idx + inner_block_idx).unwrap()
                        >> 4 * (step as usize))
                        & 0x0f;
                    let nibble_error_delta: i8 = ((nibble ^ 8) as i8) - 8;
                    let nibble_prediction = (predicted_sample
                        + (decode_ctx.channels[ctx_idx].delta as i32 * nibble_error_delta as i32))
                        .clamp(i16::MIN as i32, i16::MAX as i32);
                    if let AudioData::I16(ref mut samples) = raw_data {
                        samples.push(nibble_prediction as i16);
                    }
                    decode_ctx.channels[ctx_idx].delta = ((decode_ctx.channels[ctx_idx].delta
                        as i32
                        * ADAPTATION_TABLE[nibble as usize])
                        / ADPCM_FIXED_ADAPTATION_BASE)
                        .max(16) as i16;
                    decode_ctx.channels[ctx_idx].sample2 = decode_ctx.channels[ctx_idx].sample1;
                    decode_ctx.channels[ctx_idx].sample1 = nibble_prediction as i16;
                }
                inner_block_idx += 1;
            }
            starting_idx += inner_block_idx;
        }
        Ok(raw_data)
    }
    fn parse_fmt_metadata(
        &self,
        data: &[u8],
    ) -> Result<super::base::FmtMetadata, crate::parser::errors::AudioParserError> {
        let mut metadata = self.parse_base_fmt_metadata(data).unwrap();
        if metadata.bits_per_sample != ADPCM_BITS_PER_SAMPLE {
            return Err(InvalidFileHeaderError(format!(
                "Mismatch of the bits per sample: Got: {}, expected: {}",
                metadata.bits_per_sample, ADPCM_BITS_PER_SAMPLE
            )));
        }
        let calculated_block_align = Self::calculate_adpcm_block_align(
            metadata.sample_rate_per_second,
            metadata.number_of_channels,
        );
        if calculated_block_align != metadata.block_align {
            return Err(InvalidFileHeaderError(format!(
                "Mismatch of the block align: Got: {}, expected: {}",
                calculated_block_align, metadata.block_align
            )));
        }
        metadata.extra_bytes_size = Some(convert_to_number(data, 16, 18).unwrap());
        if metadata.extra_bytes_size.unwrap_or(0) < 32 {
            return Err(InvalidFileHeaderError(format!(
                "Mismatch of the Extra bytes size: Got: {}, expected >= 32",
                metadata.extra_bytes_size.unwrap_or(0)
            )));
        }
        metadata.samples_per_block = Some(convert_to_number(data, 18, 20).unwrap());
        let calculated_samples_per_block = Self::calculate_adpcm_samples_per_block(
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
        }
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
        metadata.coefficient_count = Some(convert_to_number(data, 20, 22).unwrap());
        if metadata.coefficient_count.unwrap() > 0 {
            let limit = 4 * metadata.coefficient_count.unwrap() as usize;
            metadata.coefficients = Some(data[22..22 + limit].to_vec());
        }
        Ok(metadata)
    }
}

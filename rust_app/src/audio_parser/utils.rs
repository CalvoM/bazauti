use crate::audio_parser::errors::AudioParserError;

pub fn fixed_string(data: &[u8]) -> String {
    let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
    String::from_utf8_lossy(&data[..end]).into_owned()
}

pub trait FromLeBytes: Sized {
    fn from_le_bytes(data: &[u8]) -> Result<Self, AudioParserError>;
}

impl FromLeBytes for i16 {
    fn from_le_bytes(data: &[u8]) -> Result<Self, AudioParserError> {
        let bytes: [u8; 2] = data
            .try_into()
            .map_err(|_| AudioParserError::InvalidByteCount {
                expected: 2,
                actual: data.len(),
            })?;

        Ok(i16::from_le_bytes(bytes))
    }
}

impl FromLeBytes for u16 {
    fn from_le_bytes(data: &[u8]) -> Result<Self, AudioParserError> {
        let bytes: [u8; 2] = data
            .try_into()
            .map_err(|_| AudioParserError::InvalidByteCount {
                expected: 2,
                actual: data.len(),
            })?;

        Ok(u16::from_le_bytes(bytes))
    }
}

impl FromLeBytes for u32 {
    fn from_le_bytes(data: &[u8]) -> Result<Self, AudioParserError> {
        let bytes: [u8; 4] = data
            .try_into()
            .map_err(|_| AudioParserError::InvalidByteCount {
                expected: 4,
                actual: data.len(),
            })?;

        Ok(u32::from_le_bytes(bytes))
    }
}

impl FromLeBytes for u64 {
    fn from_le_bytes(data: &[u8]) -> Result<Self, AudioParserError> {
        let bytes: [u8; 8] = data
            .try_into()
            .map_err(|_| AudioParserError::InvalidByteCount {
                expected: 8,
                actual: data.len(),
            })?;

        Ok(u64::from_le_bytes(bytes))
    }
}

pub fn convert_to_number<T: FromLeBytes>(
    data: &[u8],
    lower_limit: usize,
    upper_limit: usize,
) -> Result<T, AudioParserError> {
    let bytes = data
        .get(lower_limit..upper_limit)
        .ok_or_else(|| AudioParserError::InputAudioFileError("invalid byte range".to_string()))?;

    T::from_le_bytes(bytes)
}

#[repr(u16)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionCode {
    #[default]
    Unknown = 0,
    Pcm = 1,
    Adpcm = 2,
    IeeeFloat = 3,
    G711Alaw = 6,
    G711Ulaw = 7,
    ImaAdpcm = 17,
    Gsm610 = 49,
    DolbyAc2 = 48,
    MsMpeg = 80,
    Mp3 = 85,
    Aac = 255,
    Flac = 61868,
    Extensible = 65534,
}

impl std::fmt::Display for CompressionCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Unknown => "Unknown",
            Self::Pcm => "Microsoft PCM (Uncompressed)",
            Self::Adpcm => "Microsoft ADPCM",
            Self::IeeeFloat => "Microsoft IEEE Float",
            Self::G711Alaw => "ITU G.711 a-law",
            Self::G711Ulaw => "ITU G.711 u-law",
            Self::ImaAdpcm => "Intel IMA/DVI ADPCM",
            Self::Gsm610 => "Microsoft GSM610",
            Self::DolbyAc2 => "Dolby AC2",
            Self::MsMpeg => "Microsoft MPEG",
            Self::Mp3 => "MP3",
            Self::Aac => "AAC",
            Self::Flac => "Free Lossless Audio Codec FLAC",
            Self::Extensible => "Extensible",
        };

        write!(f, "{}", value)
    }
}

pub fn parse_compression_code(code: u16) -> CompressionCode {
    match code {
        1 => CompressionCode::Pcm,
        2 => CompressionCode::Adpcm,
        3 => CompressionCode::IeeeFloat,
        6 => CompressionCode::G711Alaw,
        7 => CompressionCode::G711Ulaw,
        17 => CompressionCode::ImaAdpcm,
        49 => CompressionCode::Gsm610,
        48 => CompressionCode::DolbyAc2,
        80 => CompressionCode::MsMpeg,
        85 => CompressionCode::Mp3,
        255 => CompressionCode::Aac,
        61868 => CompressionCode::Flac,
        65534 => CompressionCode::Extensible,
        _ => CompressionCode::Unknown,
    }
}

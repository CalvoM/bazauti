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
    DolbyAc2 = 48,
    Gsm610 = 49,
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

impl From<u16> for CompressionCode {
    fn from(code: u16) -> Self {
        match code {
            1 => Self::Pcm,
            2 => Self::Adpcm,
            3 => Self::IeeeFloat,
            6 => Self::G711Alaw,
            7 => Self::G711Ulaw,
            17 => Self::ImaAdpcm,
            48 => Self::DolbyAc2,
            49 => Self::Gsm610,
            80 => Self::MsMpeg,
            85 => Self::Mp3,
            255 => Self::Aac,
            61868 => Self::Flac,
            65534 => Self::Extensible,
            _ => Self::Unknown,
        }
    }
}

#[repr(u16)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListInfoId {
    #[default]
    Unknown = 0,
    Iarl = 1,
    Iart = 2,
    Icms = 3,
    Icmt = 4,
    Icop = 5,
    Icrd = 6,
    Icrp = 7,
    Idim = 8,
    Idpi = 9,
    Ieng = 10,
    Ignr = 11,
    Ikey = 12,
    Ilgt = 13,
    Imed = 14,
    Inam = 15,
    Iplt = 16,
    Iprd = 17,
    Isbj = 18,
    Isft = 19,
    Ishp = 20,
    Isrc = 21,
    Isrf = 22,
    Itch = 23,
}

impl std::fmt::Display for ListInfoId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Unknown => "Unknown",
            Self::Iarl => "Archival Location",
            Self::Iart => "Artist",
            Self::Icms => "Commissioned",
            Self::Icmt => "Comments",
            Self::Icop => "Copyright",
            Self::Icrd => "Creation date",
            Self::Icrp => "Cropped",
            Self::Idim => "Dimensions",
            Self::Idpi => "Dots Per Inch",
            Self::Ieng => "Engineer",
            Self::Ignr => "Genre",
            Self::Ikey => "Keywords",
            Self::Ilgt => "Lightness",
            Self::Imed => "Medium",
            Self::Inam => "Name",
            Self::Iplt => "Palette Setting",
            Self::Iprd => "Product",
            Self::Isbj => "Subject",
            Self::Isft => "Software",
            Self::Ishp => "Sharpness",
            Self::Isrc => "Source",
            Self::Isrf => "Source Form",
            Self::Itch => "Technician",
        };
        write!(f, "{}", value)
    }
}

impl From<[u8; 4]> for ListInfoId {
    fn from(id: [u8; 4]) -> Self {
        match &id {
            b"IARL" => Self::Iarl,
            b"IART" => Self::Iart,
            b"ICMS" => Self::Icms,
            b"ICMT" => Self::Icmt,
            b"ICOP" => Self::Icop,
            b"ICRD" => Self::Icrd,
            b"ICRP" => Self::Icrp,
            b"IDIM" => Self::Idim,
            b"IDPI" => Self::Idpi,
            b"IENG" => Self::Ieng,
            b"IGNR" => Self::Ignr,
            b"IKEY" => Self::Ikey,
            b"ILGT" => Self::Ilgt,
            b"IMED" => Self::Imed,
            b"INAM" => Self::Inam,
            b"IPLT" => Self::Iplt,
            b"IPRD" => Self::Iprd,
            b"ISBJ" => Self::Isbj,
            b"ISFT" => Self::Isft,
            b"ISHP" => Self::Ishp,
            b"ISRC" => Self::Isrc,
            b"ISRF" => Self::Isrf,
            b"ITCH" => Self::Itch,
            _ => Self::Unknown,
        }
    }
}

impl ListInfoId {
    pub const fn as_bytes(self) -> [u8; 4] {
        match self {
            Self::Unknown => *b"UNKN",
            Self::Iarl => *b"IARL",
            Self::Iart => *b"IART",
            Self::Icms => *b"ICMS",
            Self::Icmt => *b"ICMT",
            Self::Icop => *b"ICOP",
            Self::Icrd => *b"ICRD",
            Self::Icrp => *b"ICRP",
            Self::Idim => *b"IDIM",
            Self::Idpi => *b"IDPI",
            Self::Ieng => *b"IENG",
            Self::Ignr => *b"IGNR",
            Self::Ikey => *b"IKEY",
            Self::Ilgt => *b"ILGT",
            Self::Imed => *b"IMED",
            Self::Inam => *b"INAM",
            Self::Iplt => *b"IPLT",
            Self::Iprd => *b"IPRD",
            Self::Isbj => *b"ISBJ",
            Self::Isft => *b"ISFT",
            Self::Ishp => *b"ISHP",
            Self::Isrc => *b"ISRC",
            Self::Isrf => *b"ISRF",
            Self::Itch => *b"ITCH",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_string_non_null() {
        let test_payload: [u8; 5] = [72, 101, 108, 108, 111];
        let ret_val = fixed_string(&test_payload);
        assert_eq!(ret_val, String::from("Hello"));
    }

    #[test]
    fn test_fixed_string_with_null() {
        let test_payload: [u8; 7] = [72, 101, 108, 108, 0, 0, 111];
        let ret_val = fixed_string(&test_payload);
        assert_eq!(ret_val, String::from("Hell"));
    }

    #[test]
    fn test_convert_to_number_u16() -> Result<(), AudioParserError> {
        let test_payload: [u8; 6] = [2, 1, 5, 6, 7, 8];
        let ret_val = convert_to_number::<u16>(&test_payload, 0, 2)?;
        assert_eq!(ret_val, 258);
        Ok(())
    }
    #[test]
    fn test_convert_to_number_i16() -> Result<(), AudioParserError> {
        let test_payload: [u8; 6] = [0, 255, 5, 6, 7, 8];
        let ret_val = convert_to_number::<i16>(&test_payload, 0, 2)?;
        assert_eq!(ret_val, -256);
        Ok(())
    }
    #[test]
    fn test_convert_to_number_u32() -> Result<(), AudioParserError> {
        let test_payload: [u8; 6] = [2, 1, 1, 0, 0, 8];
        let ret_val = convert_to_number::<u32>(&test_payload, 1, 5)?;
        assert_eq!(ret_val, 257);
        Ok(())
    }
    #[test]
    fn test_convert_to_number_u64() -> Result<(), AudioParserError> {
        let test_payload: [u8; 12] = [2, 1, 5, 6, 1, 8, 0, 0, 0, 0, 0, 0];
        let ret_val = convert_to_number::<u64>(&test_payload, 4, 12)?;
        assert_eq!(ret_val, 2049);
        Ok(())
    }
}

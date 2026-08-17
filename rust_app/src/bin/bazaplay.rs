use rust_app::audio_parser::wav_parser::{self, WAVParser};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = WAVParser::new("audio_files/CC_0101.WAV");
    let _ = parser.parse()?;
    println!("{:#?}", parser.metadata());
    Ok(())
}

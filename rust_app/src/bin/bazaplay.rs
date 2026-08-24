use rust_app::audio_parser::wav_parser::WAVParser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = WAVParser::new("audio_files/he-man.wav");
    let _ = parser.parse()?;
    println!("{:#?}", parser.metadata());
    Ok(())
}

use rust_app::audio_parser::wav_parser::WAVParser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = WAVParser::new("audio_files/842503__icheytc__william-blake-london.wav");
    let _ = parser.parse()?;
    parser.render();
    Ok(())
}

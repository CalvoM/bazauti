#[cfg(feature = "waveform")]
use bazauti::parser::wav::WAVParser;

#[cfg(feature = "waveform")]
use bazauti::waveform::plot::WaveformRenderer;

#[cfg(feature = "waveform")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = WAVParser::new("audio_files/pcm_8bit_mono.wav");
    let _ = parser.parse()?;
    let renderer = WaveformRenderer::new(parser.context());
    renderer.render(0.0, 1.5);
    Ok(())
}

#[cfg(not(feature = "waveform"))]
fn main() {
    eprintln!("bazawaveform requires the \"waveform\" feature: rebuild with `--features waveform`");
    std::process::exit(1);
}

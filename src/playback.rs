#[cfg(feature = "playback")]
use cpal::traits::{DeviceTrait, HostTrait};

#[cfg(feature = "playback")]
pub fn play() {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("Output devices not found");
    let mut supported_configs_range = device
        .supported_output_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range
        .next()
        .expect("no supported config?!")
        .with_max_sample_rate();
    dbg!(device);
    dbg!(supported_config);
}

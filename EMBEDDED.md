# Embedded support roadmap

`bazauti` can become a capable embedded WAV parser, but its portable decoding logic should be separated from desktop conveniences.

## Priorities

1. Make the core library `no_std`.
2. Accept caller-provided bytes rather than named files.
3. Decode incrementally into caller-provided buffers.
4. Treat malformed input as an error, never a panic.

## Make the core `no_std`

- Add `#![cfg_attr(not(feature = "std"), no_std)]` to the library root.
- Make `std` an optional default feature, and use `alloc` only behind a separate feature.
- Remove `plotters` from the core dependency graph. Its rendering, image, font, filesystem, and system-library dependencies are unsuitable for microcontrollers.
- Put waveform rendering in a `std`-only example or a separate `bazauti-tools` crate.
- `thiserror` is already pinned at 2.0.20 (see `Cargo.lock`), which supports `default-features = false` for `core`-only error handling (it relies on `core::error::Error`, stable since Rust 1.81) — set that once the `std` feature is optional, and confirm with the `thumbv7em-none-eabihf` CI build below rather than assuming it.

## Parse caller-provided bytes, not file paths

`WAVParser` currently stores a `String` path and calls `fs::read()` itself. On embedded systems, WAV bytes may come from flash, an SD-card driver, DMA, UART, or a network buffer.

The core API should instead look like:

```rust
let wav = WavReader::parse(bytes)?;
let data = wav.data();
```

Provide filesystem and stream adapters separately. A `std` adapter can accept a `std::fs::File`; an embedded adapter can use `embedded_io::Read`.

## Decode with fixed-size buffers

The current implementation retains both WAV data and decoded samples in `Vec`s. That causes unbounded and unpredictable allocation.

Use a block-at-a-time API instead:

```rust
let frames_written = decoder.decode_next_block(&mut pcm_buffer)?;
audio_sink.write(&pcm_buffer[..frames_written]);
```

For ADPCM, retain only decoder state and decode one WAV block at a time. This makes SD-card playback feasible with a fixed-size RAM buffer.

## Avoid heap-heavy metadata representations

Prefer these replacements in the core API:

- `String` -> borrowed `&str`/`&[u8]`, or a fixed-capacity buffer when the caller needs ownership.
- `Vec<u8>` -> byte slices or explicit data ranges.
- `HashMap<String, String>` -> an iterator over LIST entries, or a fixed caller-provided collection.
- `Box<dyn Any>` -> typed structs or enums.

Most embedded playback needs only `fmt ` metadata and the `data` chunk. BEXT, LIST, and CUE parsing should be optional features.

## Return errors for malformed input

Do not use `unwrap`, `expect`, direct unchecked slices, or `unreachable!()` on file data. A truncated or malicious WAV must return an error rather than panic the device.

Use checked slices and checked arithmetic while traversing chunks, and enforce documented limits for channel count, metadata length, chunk count, and block size.

Confirmed panic sites in the current parser (`src/audio_parser/wav_parser.rs`), not exhaustive:

- Line 148: `(file_size - 1).try_into().unwrap()` underflows if `file_size == 0`.
- Lines 126-178: every chunk header/slice (`file_data[start_idx..start_idx + 4]`, etc.) trusts the file's own declared sizes with no check against the actual buffer length — a truncated file panics instead of erroring.
- Line 486 (`decode_ctx.channels[0].predictor * 4`, same pattern at 500): `u8` multiply overflows in debug builds and wraps silently in release if `predictor` isn't validated against `coefficient_count` first.
- ~40 other `.unwrap()` calls on `convert_to_number`, `try_into`, and `from_utf8` throughout `parse_fmt_metadata`/`parse_bext_metadata`/`parse_list_info_metadata`.
- `#![allow(warnings)]` at the top of the file (line 1) is currently suppressing lints that would otherwise flag some of this (unused `unreachable!()` arms, etc.) — remove it once the panic-safety pass lands so real issues surface again.

## Use allocation-free errors

The core error type should not require formatted `String`s or `std`:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Truncated,
    InvalidRiff,
    InvalidWave,
    InvalidChunkSize,
    UnsupportedFormat,
    UnsupportedChannels,
    InvalidFormat,
    OutputTooSmall,
}
```

Host-only wrappers can add detailed diagnostics where useful.

## Avoid floating point in validation

`calculate_adpcm_avg_bytes_per_second()` currently uses `f32`. Use integer arithmetic with a documented rounding rule, reducing code size and avoiding an FPU requirement on small targets.

## Remove parser side effects

The parser currently writes `./data.bytes` (`wav_parser.rs:169`, inside the `data` chunk branch of `parse()`), and ADPCM decoding calls `dbg!(decode_ctx)` (`wav_parser.rs:554`, end of `parse_audio_adpcm_unified`). Library code should not write files or emit debug output. Let callers select their logging and persistence behavior.

## Define the supported codec subset

The metadata recognizes many WAV compression codes, but decoding currently supports only PCM (effectively mono/stereo 8-bit and 16-bit) and Microsoft ADPCM (up to stereo). IMA ADPCM metadata is parsed but not decoded.

Expose only formats that are actually decoded, return `UnsupportedFormat` for the rest, and enforce channel limits early.

## Package and test for embedded use

- Rename the crate from `rust_app` to `bazauti`.
- Add a license, API documentation, and a minimum supported Rust version.
- Add fixtures and fuzz tests for truncated chunks, odd padding, invalid sizes, and ADPCM decoder state.
- Add CI builds such as:

  ```sh
  cargo build --no-default-features --target thumbv7em-none-eabihf
  ```

- Measure flash and RAM usage on at least one target board.

## Suggested feature layout

```toml
[features]
default = ["std", "alloc"]
std = ["alloc"]
alloc = []
metadata = ["alloc"]
render = ["std", "dep:plotters"]
```

The target architecture is a small `no_std` parser that reports WAV structure and streams decoded frames, with optional `alloc` convenience APIs and a separate host layer for files and waveform rendering.

## Immediate fixes

Address bounds checking and byte-slice input first. The current parser can panic on malformed input and writes `data.bytes` as a side effect; both are unsuitable for a reusable embedded library.

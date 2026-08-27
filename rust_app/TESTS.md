• # WAV Parser Test Iteration

  ## Iteration 1: Core Helpers

  - [ ] Add unit tests for `fixed_string`.
  - [ ] Add unit tests for `convert_to_number`.
  - [ ] Add unit tests for `parse_compression_code`.
  - [ ] Add unit tests for `parse_list_info_id`.

  ## Iteration 2: PCM Parsing

  - [ ] Add parser tests for WAV header validation.
  - [ ] Add parser tests for `fmt ` chunk parsing.
  - [ ] Cover PCM metadata validation for:
    - [ ] 8-bit mono
    - [ ] 8-bit stereo
    - [ ] 16-bit mono
    - [ ] 16-bit stereo
  - [ ] Add fixture-based end-to-end tests for the existing PCM WAV files.

  ## Iteration 3: Metadata Chunks

  - [ ] Add tests for `bext` chunk parsing.
  - [ ] Add tests for `LIST/INFO` parsing.
  - [ ] Verify odd-sized LIST subchunks are handled with padding.
  - [ ] Add tests for `fact` parsing.
  - [ ] Add tests for `cue` parsing as a no-op until implemented.

  ## Iteration 4: Error Handling

  - [ ] Add tests for invalid `RIFF` headers.
  - [ ] Add tests for invalid `WAVE` signatures.
  - [ ] Add tests for truncated or undersized chunks.
  - [ ] Add tests for inconsistent `fmt ` metadata.
  - [ ] Add tests for unsupported channel counts where render paths are used.

  ## Iteration 5: Rendering Smoke Tests

  - [ ] Add smoke tests that parse PCM data and call `render()`.
  - [ ] Keep image assertions minimal.
  - [ ] Prefer checking that rendering completes and writes output rather than comparing pixels.

  ## Iteration 6: ADPCM Follow-Up

  - [ ] Add tests that cover the current ADPCM and IMA ADPCM metadata branches.
  - [ ] Document unsupported decode behavior until the audio decode paths are implemented.
  - [ ] Add regression tests before implementing full ADPCM decoding.

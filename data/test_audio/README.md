# Test Audio

Reference notes for small local WAV sets used for Riotbox TUI smoke runs.

Audio files under `data/` are ignored by Git. Keep only lightweight documentation and download notes in the repo, then place the actual audio files locally beside these notes when needed.

Sources:

- MusicRadar `SampleRadar: 494 free techno drum and FX samples`
  - https://www.musicradar.com/news/tech/sampleradar-494-free-techno-drum-and-fx-samples-550889
  - direct archive used: `https://cdn.mos.musicradar.com/audio/samples/musicradar-techno-drum-fx-samples.zip`
- MusicRadar `SampleRadar: 267 free deep house samples`
  - https://www.musicradar.com/music-tech/samples/sampleradar-267-free-deep-house-samples
  - direct archive used: `https://cdn.mos.musicradar.com/audio/musicradar-deep-house-samples.zip`

Selected examples:

- `examples/Beat03_130BPM(Full).wav`
  - clear techno full loop
- `examples/Beat08_128BPM(Full).wav`
  - slightly steadier 128 BPM techno loop
- `examples/Beat20_128BPM(Full).wav`
  - another full 128 BPM techno loop for comparison
- `examples/DH_BeatC_120-01.wav`
  - deeper house full beat loop
- `examples/DH_BeatC_KickSnr_120-01.wav`
  - simpler kick/snare-focused house loop
- `examples/DH_Fadapad_120_A.wav`
  - more tonal/pad-driven loop
- `examples/DH_RushArp_120_A.wav`
  - more melodic/arp-driven loop

These files are intended for local testing, not as repo-canonical assets.

P023 sound-excellence corpus mapping:

- `Beat03_130BPM(Full).wav`: dense-break Golden Path source.
- `Beat08_128BPM(Full).wav`: alternate dense-break diversity source.
- `DH_BeatC_KickSnr_120-01.wav`: sparse-drums / bass-pressure source.
- `DH_RushArp_120_A.wav`: tonal-riff / hook source.
- `DH_Fadapad_120_A.wav`: pad-noise source and degraded timing-policy source.
- `Beat20_128BPM(Full).wav`: weak-source / ordinary-loop regression source.

The machine-readable contract is
`docs/benchmarks/sound_excellence_source_corpus_v1.json`.

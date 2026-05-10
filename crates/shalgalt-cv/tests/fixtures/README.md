# `shalgalt-cv` test fixtures

This directory hosts **synthetic** scanned-OMR raster images used by the
integration tests. They are created on-the-fly in Rust (see
`tests/common/synth.rs`) instead of being committed as binary blobs, for two
reasons:

1. Real scans of the Mongolian-standard preset are not available in our local
   development environment. The closest substitute is to render the preset
   with a known answer pattern, then compose a believable scan artifact on
   top.
2. Binary PNG fixtures bloat the repo, drift silently when CV defaults change,
   and are hard to review. A Rust generator keeps the inputs deterministic and
   diffable.

## Categories

`tests/common/synth.rs` exposes three generator functions, each returning a
`Vec<u8>` containing a PNG:

| Category   | Function                       | Behaviour                                                                                              |
| ---------- | ------------------------------ | ------------------------------------------------------------------------------------------------------ |
| MFP        | `synth::mfp_quality`           | Clean rasterization of an empty Mongolian-standard sheet with light noise + four canonical ArUco IDs. |
| phone      | `synth::phone_quality`         | Same source, then perspective distortion (±5°), gaussian blur, uneven illumination.                    |
| bad        | `synth::deliberately_bad`      | Heavy noise, partial occlusion, or extreme tilt — designed to make detection fail gracefully.          |

The integration test `pipeline_integration.rs` invokes these and asserts:

- ArUco corner detection succeeds for MFP samples.
- Fill ratios for known-filled bubbles exceed 0.65 in MFP samples.
- The "bad" samples return an `AppError::BadRequest` instead of panicking.

## Real-scan validation

True real-scan validation will land in P8 hardening once representative MFP
output and phone photos are collected from a partner school. Until then, the
synthetic fixtures are the contract that locks the pipeline against
regressions.

# HDR probe (#189, never merged)

Throwaway tools behind `docs/research/hdr-exposure-and-tonemapping.md`. This
branch, `research/hdr-captures`, is never merged: it patches `praxis capture`
to write unclamped linear radiance and adds this probe crate, which applies
the candidate tonemapping operators to those captures and times the
candidate passes.

## Captures

From the repository root, after `scripts/fetch-assets.sh`:

```sh
cargo build --release
for scene in pbr-sweep helmet orrery minification normal-tangent; do
  ./target/release/praxis capture $scene /tmp/hdr-in/$scene.f32 1280x720
done
```

A `.f32` file is the width and height as little-endian `u32`, then linear
RGBA `f32` texels, rows top first: the main pass's output into an
`Rgba32Float` target with blending off, before any clamp.

## Images and timings

The Tony McMapface LUT is not committed; fetch it at a pinned commit (MIT or
Apache-2.0, `h3r2tic/tony-mc-mapface`):

```sh
curl -sfLo /tmp/tony_mc_mapface.dds \
  https://raw.githubusercontent.com/h3r2tic/tony-mc-mapface/0f249d366c9e960aa9828818786a6d5900fd85d9/shader/tony_mc_mapface.dds
shasum -a 256 /tmp/tony_mc_mapface.dds  # a492a9f4dd981e05ff5f1edbf6775ac26b8c8bbeaaecc8b48207873bf46dbdf0
cd research/hdr-probe
TONY_LUT=/tmp/tony_mc_mapface.dds cargo run --release --offline -- images /tmp/hdr-in /tmp/hdr-out
TONY_LUT=/tmp/tony_mc_mapface.dds cargo run --release --offline -- bench
cargo run --release --offline -- overflow
```

`images` writes one PNG per input and operator (a synthetic hue and exposure
sweep plus every capture), a dim gradient without and with the static dither,
and the resolve comparison: 4x MSAA box-resolved, weighted, and two 64-sample
references averaged in scene-linear and in display-linear space.

`bench` times each pass at 2560×1440: 45 submissions of 8 back-to-back
repetitions each, timed from the first repetition's first pass to the last
repetition's last pass and divided by 8, waited on one by one and resolved
after the last, the first 5 submissions left out, after 2 s of warm-up.

`overflow` writes values past 65504 to an `Rgba16Float` attachment, at 1x and
as one sample of a 4x hardware resolve, and prints what is stored.

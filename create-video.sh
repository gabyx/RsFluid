#!/bin/bash
set -u
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"

frameRate="${1?'First argument: Framerate.'}"
shift 1

timestep=$(echo "scale=3; 1.0/$frameRate" | bc)
frameRateVideo="$frameRate"

cargo run --release --bin rustofluid -- \
    -e 15.0 \
    -t "$timestep" \
    --show-progress \
    --incompress-iters 150 \
    --dim "1920, 640" \
    --plot-dim "1920, 640" \
    --plot-velocity \
    --plot-pressure \
    --plot-masked-pressure \
    --plot-masked-velocity \
    "$@"

cd "$DIR" &&
    ffmpeg -y -framerate "$frameRateVideo" -pattern_type glob -i 'frames/frame-press-*.png' \
        -c:v libx264 -vf "pad=ceil(iw/2)*2:ceil(ih/2)*2" -pix_fmt yuv420p video-press.mp4

cd "$DIR" &&
    ffmpeg -y -framerate "$frameRateVideo" -pattern_type glob -i 'frames/frame-vel-*.png' \
        -c:v libx264 -vf "pad=ceil(iw/2)*2:ceil(ih/2)*2" -pix_fmt yuv420p video-vel.mp4

cd "$DIR" &&
    ffmpeg -y -framerate "$frameRateVideo" -pattern_type glob -i 'frames/frame-smoke-*.png' \
        -c:v libx264 -vf "pad=ceil(iw/2)*2:ceil(ih/2)*2" -pix_fmt yuv420p video-smoke.mp4

rm -rf "$DIR/frames"

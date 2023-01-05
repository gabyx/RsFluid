#!/bin/bash
set -u
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"

frameRate="${1?'First argument: Framerate.'}"
timestep=$(echo "scale=3; 1.0/$frameRate" | bc)
frameRateVideo=$(echo "$frameRate*1.5" | bc)

cargo run --release --bin rustofluid -- -e 120 -t "$timestep" --incompress-iters 40 --dim "400,200"

cd "$DIR" &&
    ffmpeg -y -framerate "$frameRateVideo" -pattern_type glob -i 'frames/frame-press-*.png' \
        -c:v libx264 -vf "pad=ceil(iw/2)*2:ceil(ih/2)*2" -pix_fmt yuv420p video-press.mp4

cd "$DIR" &&
    ffmpeg -y -framerate "$frameRateVideo" -pattern_type glob -i 'frames/frame-vel-*.png' \
        -c:v libx264 -vf "pad=ceil(iw/2)*2:ceil(ih/2)*2" -pix_fmt yuv420p video-vel.mp4

rm -rf "$DIR/frames"

#!/bin/bash
set -u
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"

frameRate="${1?'First argument: Framerate.'}"

cd "$DIR" &&
    ffmpeg -framerate "$frameRate" -pattern_type glob -i 'frames/*.png' \
        -c:v libx264 -pix_fmt yuv420p video.mp4

rm -rf "$DIR/frames"

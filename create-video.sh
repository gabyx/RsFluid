#!/bin/bash
set -u
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"

frameRate="${1?'First argument: Framerate.'}"

cd "$DIR" &&
    ffmpeg -y -framerate "$frameRate" -pattern_type glob -i 'frames/frame-press-*.png' \
        -c:v libx264 -vf "pad=ceil(iw/2)*2:ceil(ih/2)*2" -pix_fmt yuv420p video-press.mp4

cd "$DIR" &&
    ffmpeg -y -framerate "$frameRate" -pattern_type glob -i 'frames/frame-vel-*.png' \
        -c:v libx264 -vf "pad=ceil(iw/2)*2:ceil(ih/2)*2" -pix_fmt yuv420p video-vel.mp4

rm -rf "$DIR/frames"

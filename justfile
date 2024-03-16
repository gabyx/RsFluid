set positional-arguments
set shell := ["bash", "-cue"]
root_dir := justfile_directory()

build *args:
    cd "{{root_dir}}" && \
      cargo build "${@:1}"

run *args:
    cd "{{root_dir}}" && \
      cargo run "${@:1}"

clean:
  cargo clean

video frames="30":
    cd "{{root_dir}}" && \
      ./tools/create-video.sh "{{frames}}"

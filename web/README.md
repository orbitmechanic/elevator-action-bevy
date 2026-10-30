# Elevator Action — web shell

This directory contains the static page that hosts the Bevy WebGL build.

`tools/build_web.sh` compiles the game for `wasm32-unknown-unknown`, runs
`wasm-bindgen`, copies the runtime assets, and stages a deployable folder in
`web/dist/`.

```bash
# once
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129

# every build
tools/build_web.sh
python3 -m http.server 8123 --directory web/dist
# open http://localhost:8123/
```

The shell uses a 960×720 canvas with CSS scaling, a click-to-focus overlay
(also unlocks browser audio), and a fullscreen button. The game needs WebGL 2
and a keyboard.

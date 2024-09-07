# CHIP-8 in Rust in WASM

## Setup
- install `sdl2` via Homebrew

## Play
```
make run-desktop GAME_PATH=$(pwd)/games/TETRIS

```

## Viewing the Games in Hex
- open them up and do a `:%!xxd` to get them into a hex view and a `:%!xxd -r` to get them back into binary

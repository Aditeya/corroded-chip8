# Corroded Chip 8

This has some more work TODO:
- [ ] Add a byte/op code language parser
- [ ] The screen renders poorly which i would like to fix.
- [ ] Moving from SDL to macroquad for a (more) pure rust implmentatation.
- [ ] Generate Sound
- [ ] support above in WASM ???

A Chip 8 Emulator written in rust.
I followed this [guide](https://github.com/aquova/chip8-book) to make it.

chip8 games: https://www.zophar.net/pdroms/chip8/chip-8-games-pack.html

## Running

### SDL

```sh
$ just run roms/MAZE
```

### Wasm

make sure you have wasm-pack and miniserve installed.

```sh
$ just run-wasm
```

# Puzzle 15 game

To proceed you need a Rust compiler installed on your machine: https://rustup.rs

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Run it

```sh
cargo run --bin puzzle_15
```

Run without shuffling: 
```sh
cargo run --bin puzzle_15 -- --no-shuffle
```

## Test it

```sh
cargo test
```

## Try it in your browser

https://anti-social.github.io/puzzle-15/

## Run web version locally

Install Rust WASM bundler:

```sh
cargo install trunk
```

Run it:

```sh
cd web
trunk serve --release --open
```

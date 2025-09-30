# Oxide Mongo

![Oxide Mongo icon](icons/oxide_mongo_256x256.png)

Oxide Mongo is a desktop GUI client for MongoDB, written by [Rust](https://www.rust-lang.org/)
 built with [Iced](https://github.com/iced-rs/iced).

TODO

## Features

- TODO

## Getting Started

### Prerequisites

- Rust toolchain (stable channel) installed via [rustup](https://rustup.rs/)
- Optional: `cargo install cargo-watch` for iterative development

### Build & Run

```bash
cargo run
```

### Format & Lint

```bash
cargo fmt
cargo clippy -- -D warnings
```

### Test

```bash
cargo test
```

## Project Structure

TODO

```
```

## Knowing Issues

1. After computer's hibernate wake up  the application not working, main window freeze. This is issue of GPU subsystem of Linux and not proglem directly Oxide Mongo. This issue happens in other differents software, which uses similar techmologies.

## Contributing

1. Fork the repository and create a feature branch.
2. Keep the code formatted (`cargo fmt`) and free of warnings (`cargo clippy -- -D warnings`).
3. Run the test suite (`cargo test`).
4. Submit a pull request with a clear description of the changes.

Bug reports and feature ideas are welcome through the GitHub issue tracker.

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

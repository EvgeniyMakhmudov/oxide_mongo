# Oxide Mongo

# About

![Oxide Mongo icon](assests/icons/oxide_mongo_256x256.png)

Oxide Mongo is a fast, lightweight, cross-platform GUI client for MongoDB.
Inspired by Robomongo/Robo3T and built in [Rust](https://www.rust-lang.org/) with
[Iced](https://github.com/iced-rs/iced), it focuses on everyday work: browsing data,
running queries, and managing collections without extra overhead.

All source code in this project was generated with the assistance of AI tools (OpenAI Codex, GPT-4/5, Google Gemini).

To ensure the project is safe for production use, a strict integration test is included. The test verifies that the system behaves exactly as expected and nothing more.
The test logic, expected results, and acceptance criteria were defined and reviewed by a human under direct supervision.

This approach ensures production readiness regardless of the code generation method.

## Features

- Connection profiles with optional authentication and SSH tunneling.
- Mongo shell-like query editor with commonly used commands.
- Change Streams (`watch`) that stream results until the limit is reached.
- Results view in tree-like table or JSON-like text.
- Database and collection actions: create, drop, rename, stats, indexes.
- Replica set helpers (`rs.*`) and admin commands (`db.adminCommand`).
- Theme and font customization.
- Internalization UI.

## Screenshots

![1](https://github.com/user-attachments/assets/edb622c7-0f8b-4119-a0c1-d64b22b4e3ec)
![2](https://github.com/user-attachments/assets/286a857b-7d79-44fc-b76c-abc2c56c80a1)
![3](https://github.com/user-attachments/assets/8da5b109-98c2-4344-b5f3-f6227b066b8c)
![4](https://github.com/user-attachments/assets/61fe5d10-9b9a-403a-bd8c-8961d19dbe1f)

## Download

Prebuilt binaries are published on GitHub Releases:
https://github.com/EvgeniyMakhmudov/oxide_mongo/releases


## Getting Started

### Prerequisites

- Rust toolchain (stable) via [rustup](https://rustup.rs/)
- A MongoDB instance to connect to

On Linux you may need X11/Wayland development packages (see CI workflow for the list).

### Build and Run

```bash
cargo run
```

### Format and Lint

```bash
cargo fmt
cargo check
```

### Tests

```bash
# unit tests
cargo test

# integration tests (requires a running MongoDB instance)
OXIDE_MONGO_TEST_URI=mongodb://localhost:27017 cargo test -- --ignored
```

## Documentation

Built-in documentation is available in the app menu: Help -> Documentation.

## Known Issues

1. After waking a Linux system from hibernation, the main window may freeze.
   This is a GPU subsystem issue and affects other GUI applications as well.

## Contributing

1. Fork the repository and create a feature branch.
2. Keep the code formatted (`cargo fmt`) and free of warnings (`cargo check`).
3. Run the test suite (`cargo test`).
4. Submit a pull request with a clear description of the changes.

Bug reports and feature ideas are welcome through the GitHub issue tracker.

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

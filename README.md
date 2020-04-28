# wired

WIP: Collection of embeddable database models for Rust.

[![Documentation](https://docs.rs/wired/badge.svg)](https://docs.rs/wired)
[![Crate](https://img.shields.io/crates/v/wired.svg)](https://crates.io/crates/wired)
![Build](https://github.com/Anonyfox/wired/workflows/CI/badge.svg)

## Features

- **schema-free**: use anything that can be serialized with serde/bincode
- **portable**: every database is persisted with a single memory-mapped binary file
- **lightweight**: pure Rust implementation without many internal dependencies
- **broadly available**: works on current stable rust
- **efficient**: uses a self-managed block storage that recycles memory
- **fast**: reading and writing should both be a `O(1)` operation

## Work in Progress

This is a personal learning project and existing APIs may change any time.
Also, the format of the binary encoded files might not be compatible between
versions. Once everything is stable I will release a version 1.0 and backwards
compatibility will be guaranteed.

In the mean time, I would love to get feedback on how the databases work in
real world use cases and where some bugs are lurking. This will stabilize the
lib faster and I might learn more from your feedback!

## Available Models

- [x] Stack
- [x] Queue
- [ ] Log
- [x] Key-Value
- [ ] Document
- [ ] Graph
- [ ] Tabular
- [ ] Relational

## License

MIT

# wired

WIP: Collection of embeddable database models for Rust.

[![Documentation](https://docs.rs/wired/badge.svg)](https://docs.rs/wired)
[![Crate](https://img.shields.io/crates/v/wired.svg)](https://crates.io/crates/wired)
![Build](https://github.com/Anonyfox/wired/workflows/CI/badge.svg)

## Features

- schema-free: use anything that can be serialized with serde
- portable: every database is persisted with a single memory-mapped binary file
- lightweight: pure Rust implementation without many internal dependencies

## Available Models

- [x] Stack
- [x] Queue
- [ ] Log
- [ ] Key-Value
- [ ] Document
- [ ] Graph
- [ ] Tabular
- [ ] Relational

## Copy-On-Write

Every database type uses a "copy-on-write"-mechanism. Therefore, any time a new
record is persisted or an existing record is updated, the database file will
grow accordingly. This results in really fast write operations in exchange for
a larger-than-neccessary data file on disk. It is possible to defragment a
database file by calling the `compact()` method, though.

## License

MIT

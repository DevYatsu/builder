# builder

[![CI](https://github.com/DevYatsu/builder/actions/workflows/ci.yml/badge.svg)](https://github.com/DevYatsu/builder/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/builder.svg)](https://crates.io/crates/builder)

A minimalist, universal build utility written in Rust.

builder automatically detects the build system in your current directory and provides a unified interface for building, running, and testing projects across multiple languages and frameworks.

## Features

- Automatic environment detection.
- Minimalist console output with bracketed logging.
- Support for build, run, and test modes.
- Ninja generator support for CMake projects.
- Automatic binary execution for systems without a native run command.
- Zero configuration required.

## Supported Systems

- Rust (Cargo)
- Makefile
- CMake (with Ninja support)
- Node.js (npm)
- Go
- Docker
- Maven
- Gradle
- Zig
- .NET (C#, F#)

## Installation

### Via Script (macOS and Linux)
The fastest way to install the latest version:

```bash
curl -fsSL https://raw.githubusercontent.com/DevYatsu/builder/master/install.sh | bash
```

### From Source
```bash
git clone https://github.com/DevYatsu/builder.git
cd builder
cargo install --path .
```

## Usage

```bash
builder [OPTIONS] [DIRECTORY]
```

### Options

- -x, --run      : Build and execute the project (uses native run or binary detection).
- -t, --test     : Run project tests.
- -r, --release  : Build with release optimizations.
- -l, --list     : List all supported build systems.
- -d, --dir <D>  : Run builder inside the specified directory.
- -h, --help     : Show help information.

### Examples

Build the project in the current directory:
```bash
builder
```

Build and run a C++ CMake project in release mode:
```bash
builder -x -r
```

Run tests in a specific directory:
```bash
builder -t -d ./my-subproject
```

## Internal Testing

To run the integration tests for the builder tool itself:

```bash
cargo test
```

## License

MIT

# ribzip2 - a comprehensible bzip2 implementation

`ribzip2` is command line utility providing bzip2 compression and decompression written in pure Rust. It is currently
considered work-in-progress and lacks many features of the original implementation including

 * it has worse compression rate
 * it is slower (at least factor 2)

# Usage

Beware that `ribzip2` is WIP. If you absolutely want to, install `ribzip2` using `cargo install ribzip2`.
You can use `ribzip2 compress <FILENAME>` to compress a file and `ribzip2 decompress <FILENAME>`.
The latter will output from `file.bz2` to `file.out`. For further information use the help subcommand
and the respective help options of `compress` and `decompress`, e.g. `ribzip2 compress --help`.

# Design Goals

## Goals

 * "enterprise style" comprehensible code equipped with tests explaining the involved algorithmns
 * pure rust
 * safe code
 * state-of-the-art algorithms with optimal asymptotic performance
 * efficient multithreading
 * ergonomic cli

## Long-Term-Goals

 * ergonomic library crate
 * drop-in replacement for the bzip2 crate
 * drop-in replacement for C libbzip2
 * drop-in replacement for the C bzip2/bunzip2 binary

## Publishing

We use the crate `cargo-workspaces` for publishing releases manually. Currently, there is no automation in place.

# Contribute!

Contributions are very welcome (issues, pull-requests, and comments). Find your issue under "issues".
The code here (exlcuding samples using for compression tests) is published
under the MIT license.

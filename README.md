# ribzip2 - a comprehensible bzip2 implementation

`ribzip2` is command line utility providing bzip2 compression and decompression written in pure Rust. It is currently
considered work-in-progress and lacks many features of the original implementation including

 * it has worse compression rate
 * it is slower (at least factor 2)

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
 * drop-in replacement for C libzip2
 * drop-in replacement for the C bzip2/bunzip2 binary

# Contribute!

Contributions are very welcome (issues, pull-requests, and comments). Find your issue under "issues".
The code here (exlcuding samples using for compression tests) is published
under the MIT license.

# libribzip2 - a comprehensible pure Rust bzip2 implementation

`libribzip` attempts to be a comprehensible bzip2 implementation. It is currently WIP and
to be considered unstable (interface-wise) and incomplete.

# Features

 * pure safe-Rust implementation with no dependencies
 * multithreaded encoding
 * linear-time Burrows-Wheeler transform using SA-IS and Duval's algorithm
 * flexible computation of Huffman codes using one of
  * static global frequency tables
  * local tables computed using k-means clustering through Lloyd's algorithm

# Contributing

`libribzip2` is part of `ribzip2`, see the contribution guidelines there.

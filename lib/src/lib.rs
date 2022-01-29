//! Crate for encoding and decoding bzip2 streams
//! Currently currently has a pretty narrow interface:
//! 
//! The main interfaces are
//! 
//!  * [stream::encode_stream]
//!  * [stream::decode_stream]
mod bitwise;
mod block;
pub mod stream;
pub use block::symbol_statistics::EncodingStrategy;

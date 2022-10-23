#![no_std]
//#![deny(warnings)]
#![forbid(unsafe_code)]
//#![deny(missing_docs)]

//! Minimal compression & decompression library for embedded use
//! Implements the Heatshrink compression algorithm
//! described here <https://github.com/atomicobject/heatshrink>
//! and here <https://spin.atomicobject.com/2013/03/14/heatshrink-embedded-data-compression/>

pub mod decoder;
#[cfg(feature = "encode")]
pub mod encoder;

pub use decoder::HeatshrinkDecoder as HeatshrinkDecoder;

#[cfg(feature = "encode")]
pub use encoder::{encode, EncodeError};
/// Structure holding the configuration parameters
/// These can be tuned to improve compression ratio
/// But bbust be the same for encode() & decode()
/// calls to be able to produce the original data
//#[derive(Debug, Copy, Clone)]
#[derive(Copy,Clone)]
pub struct Config {
    pub(crate) window_sz2: u8,
    pub(crate) lookahead_sz2: u8,
}

impl Default for Config {
    fn default() -> Self {
        let window_sz2 = 11;
        let lookahead_sz2 = 4;
        Config {
            window_sz2,
            lookahead_sz2,
        }
    }
}

impl Config {
    /// Creates a enw configuration object with default values
    pub fn new(window_sz2: u8, lookahead_sz2: u8) -> Result<Self, &'static str> {
        Config::default()
            .with_window(window_sz2)
            .and_then(|c| c.with_lookahead(lookahead_sz2))
    }

    /// Modifies the configuration with a desired window size ( in range 1 - 16 )
    pub fn with_window(mut self, window_sz2: u8) -> Result<Self, &'static str> {
        if window_sz2 > 16 {
            Err("Window is too large")
        } else if window_sz2 == 0 {
            Err("Window is too small")
        } else {
            self.window_sz2 = window_sz2;
            Ok(self)
        }
    }

    /// Modifies the configuration with the desired lookahead
    pub fn with_lookahead(mut self, lookahead_sz2: u8) -> Result<Self, &'static str> {
        if lookahead_sz2 > 16 {
            Err("Window is too large")
        } else if lookahead_sz2 == 0 {
            Err("Window is too small")
        } else {
            self.lookahead_sz2 = lookahead_sz2;
            Ok(self)
        }
    }
}


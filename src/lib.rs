#![no_std]
//#![deny(warnings)]
#![forbid(unsafe_code)]
//#![deny(missing_docs)]

//! Minimal compression & decompression library for embedded use
//! Implements the Heatshrink compression algorithm
//! described here <https://github.com/atomicobject/heatshrink>
//! and here <https://spin.atomicobject.com/2013/03/14/heatshrink-embedded-data-compression/>

mod decoder;
mod encoder;

pub use decoder::HeatshrinkDecoder as HeatshrinkDecoder;

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

#[cfg(test)]
mod test {
    extern crate std;
    use super::{decoder, encoder, Config};

    fn compare(a1: &[u8], a2 : &[u8]) {
        assert_eq!(a1.len(), a2.len());
        for i in 0..a1.len()
        {
            if a1[i]!=a2[i]
            {
                std::println!("Mismatch at index {}",i);
            }
            assert_eq!(a1[i], a2[i]);
        }
    }
    fn dump(name : &str, a1: &[u8]) {
        std::println!("{}",name);
        for i in 0..a1.len()
        {
            std::print!("[{}]{:#04x}, ",i,a1[i]);
        }
        std::print!("\n");
    }
    #[test]
    fn short_decode() {
        let src = [
            // bd a0 33
            /*0*/189, 160, 51, 163,   0, 0, 0, 0,    0, 0, 0, 0,     0, 0, 0, 0,    0,
            /*17*/0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
            /*28*/199, 0, 0, 0, 0, 0, 0, 0, 
            /*36*/166, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
            /*52*/154, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ];
        let mut dst1 = [0; 100];
        let mut dst2 = [0; 100];
        
        let cfg = Config::new(11, 4).unwrap();
        
        let out1 = encoder::encode(&src, &mut dst1, &cfg).unwrap();
        std::println!("Input ({}) -> compressed {}",src.len(),out1.len());
        //
        let mut decoder = decoder::HeatshrinkDecoder::new(out1,&cfg);
        std::println!("compressed ->dst2",);
        for i in 0..src.len()
        {
            dst2[i]=decoder.next();
            std::print!("[{}] {}\n",i,dst2[i]);
        }
        let result = &dst2[..src.len()];
        dump("Src",&src);
        dump("dst",result);
        compare(&src,result);
    }
}

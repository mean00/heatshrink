
use super::Config;

// In theory OUTPUT_BUFFER_SIZE = 1<<window_size+1<<lookahead_sz2
// default value allocate about 384 bytes
// It is similar to window=7, lookahead=4
const OUTPUT_BUFFER_SIZE :  usize = 0x100;
const OUTPUT_BUFFER_SIZE_MASK :  usize = OUTPUT_BUFFER_SIZE-1;

#[derive(Copy, Clone)]
enum HSDstate {
    HSDSTagBit,          /* tag bit */
    HSDSYieldLiteral,    /* ready to yield literal byte */
    HSDSBackrefIndexMsb, /* most significant byte of index */
    HSDSBackrefIndexLsb, /* least significant byte of index */
    HSDSBackrefCountMsb, /* most significant byte of count */
    HSDSBackrefCountLsb, /* least significant byte of count */
    HSDSYieldBackref,    /* ready to yield back-reference */
    HSDSNeedMoreData,    /* End of input buffer detected */    
}

#[derive(Copy, Clone)]
pub struct HeatshrinkDecoder<'a> {
    output_count    : u16, // nb to copy
    rewind          : u16, // back ref
    state           : HSDstate,
    input_index     : usize,  // Input index
    cfg             : Config,
    input           : &'a [u8],
    bitbuffer       : u32,
    bitcount        : usize,
    
    // The output_buffer has the following structure
    // 0 ..... [head...tail]....    
    //
    output_buffer : [u8;OUTPUT_BUFFER_SIZE], // must be able to contain a full window
    output_head   : usize,
    output_tail   : usize,     
}


impl<'a> HeatshrinkDecoder<'a> {
    pub fn new(input: &'a [u8], cfg: &Config) -> Self {        
        HeatshrinkDecoder {
            output_count    : 0,
            rewind    : 0,
            state           : HSDstate::HSDSTagBit,
            input_index       : 0,
            cfg             : *cfg,
            input           : input,
            bitbuffer       : 0,
            bitcount        : 0,    
            output_buffer   : [0;OUTPUT_BUFFER_SIZE],
            output_head     : 0,
            output_tail     : 0,            
        }
    }
    pub fn reset(&mut self, input: &'a [u8]) -> bool {

        self.output_count   = 0;
        self.rewind   = 0;
        self.state          = HSDstate::HSDSTagBit;
        self.input_index    = 0;
        self.input          = input;        
        self.output_head    = 0;
        self.output_tail    = 0;
        self.bitbuffer      = 0;
        self.bitcount       = 0;    
        self.input_index    = 0;

        true
    }
    pub fn next(&mut self) -> u8 {

        loop {
            // do we have data available ?
            if self.output_head<self.output_tail
            {
                let r=self.output_buffer[self.output_head & OUTPUT_BUFFER_SIZE_MASK];
                self.output_head+=1;                                
                return r;
            }

            loop {
                self.state = match self.state {
                    HSDstate::HSDSTagBit => self.st_tag_bit(),
                    HSDstate::HSDSYieldLiteral => self.st_yield_literal(),
                    HSDstate::HSDSBackrefIndexMsb => self.st_backref_index_msb(),
                    HSDstate::HSDSBackrefIndexLsb => self.st_backref_index_lsb(),
                    HSDstate::HSDSBackrefCountMsb => self.st_backref_count_msb(),
                    HSDstate::HSDSBackrefCountLsb => self.st_backref_count_lsb(),
                    HSDstate::HSDSYieldBackref => self.st_yield_backref(),
                    HSDstate::HSDSNeedMoreData => {
                        break;
                    }                  
                };
                // println!("State: {:?} {:?}", self.state, self.bit_index);
                if self.input_index > self.input.len()    {
                    panic!("input buffer overflow");                
                }                
                if self.output_head!=self.output_tail
                {
                    break;
                }
            }
        }
    }

    fn get_bits(&mut self, count: u8) -> Option<u16> {
        let count : usize = count as usize;
        loop {
            if self.bitcount >= count
            {
                // extract on the left
                let mut r : u32 = self.bitbuffer ;
                r >>= self.bitcount-count;
                r&= (1<<count)-1;
                self.bitcount-=count;
                return Some(r as u16);
            }        
            if self.input_index > self.input.len() {
                return None;
            }
            self.bitbuffer<<=8;            
            self.bitbuffer|=self.input[self.input_index] as u32;
            self.input_index+=1;
            self.bitcount+=8;
        }
    }

    fn st_tag_bit(&mut self) -> HSDstate {
        match self.get_bits(1) {
            Some(0) => {
                if self.cfg.window_sz2 > 8 {
                    HSDstate::HSDSBackrefIndexMsb
                } else {
                    self.rewind = 0;
                    HSDstate::HSDSBackrefIndexLsb
                }
            }
            Some(_) => HSDstate::HSDSYieldLiteral,
            None => HSDstate::HSDSNeedMoreData,
        }
    }

    fn st_yield_literal(&mut self) -> HSDstate {
        let byte = match self.get_bits(8) {
            Some(b) => b,
            None => {
                return HSDstate::HSDSNeedMoreData;
            }
        };
        self.output_buffer[self.output_tail & OUTPUT_BUFFER_SIZE_MASK] = byte as u8;
        self.output_tail += 1;
        //std::print!("litteral {}\n",1);
        HSDstate::HSDSTagBit
    }

    fn st_backref_index_msb(&mut self) -> HSDstate {
        let bit_ct = self.cfg.window_sz2 - 8;
        self.rewind = match self.get_bits(bit_ct) {
            Some(idx) => idx << 8,
            None => {
                return HSDstate::HSDSNeedMoreData;
            }
        };
        HSDstate::HSDSBackrefIndexLsb
    }

    fn st_backref_index_lsb(&mut self) -> HSDstate {
        let bit_ct = self.cfg.window_sz2.min(8);
        self.rewind = match self.get_bits(bit_ct) {
            Some(idx) => self.rewind | idx,
            None => {
                return HSDstate::HSDSNeedMoreData;
            }
        };
        self.rewind += 1;
        self.output_count = 0;
        if self.cfg.lookahead_sz2 > 8 {
            HSDstate::HSDSBackrefCountMsb
        } else {
            HSDstate::HSDSBackrefCountLsb
        }
    }

    fn st_backref_count_msb(&mut self) -> HSDstate {
        let bit_ct = self.cfg.lookahead_sz2 - 8;
        self.output_count = match self.get_bits(bit_ct) {
            Some(idx) => idx << 8,
            None => {
                return HSDstate::HSDSNeedMoreData;
            }
        };
        HSDstate::HSDSBackrefIndexLsb
    }

    fn st_backref_count_lsb(&mut self) -> HSDstate {
        let bit_ct = self.cfg.lookahead_sz2.min(8);
        self.output_count = match self.get_bits(bit_ct) {
            Some(idx) => self.output_count | idx as u16,
            None => {
                return HSDstate::HSDSNeedMoreData;
            }
        };
        self.output_count += 1;
        HSDstate::HSDSYieldBackref
    }

    fn st_yield_backref(&mut self) -> HSDstate {
        /* println!(
            "Backref: idx:{}  count:{}",
            self.rewind, self.output_count
        ); */
        let count = self.output_count as usize;
        if self.output_tail <  self.rewind as usize
        {
            panic!("Rewinding too much");
        }        
        let start_in = self.output_tail - self.rewind as usize;       
        for i in 0..count {
            self.output_buffer[(self.output_tail+i+OUTPUT_BUFFER_SIZE)& OUTPUT_BUFFER_SIZE_MASK] 
                        = self.output_buffer[(start_in + i+OUTPUT_BUFFER_SIZE)& OUTPUT_BUFFER_SIZE_MASK];
        }
        //std::print!("Copy {}\n",count);
        self.output_tail +=count;                
        HSDstate::HSDSTagBit
    }
}

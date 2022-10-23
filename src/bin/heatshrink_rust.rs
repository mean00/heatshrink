/*
    Quick & Dirty CLI encoder

*/
#![allow(non_snake_case)]
#![feature(path_file_prefix)]
use clap;
use clap::Parser;
use std::fs::File;
use std::io::{Write,Read};

use heatshrink_byte::encoder;
use heatshrink_byte::decoder as hsdecoder;

use heatshrink_byte::Config as hsConfig;

#[derive(Parser, Debug)]
#[clap(author="mean00",version="0.1",about="Standalone heatshrink encoder/decoder",long_about = None)]
struct Config {
    /// Path to the font to use
    #[clap(short, long)]
    input_file: String,
    // output file
    #[clap(short, long)]
    output_file: String,
    // decode
  //  #[clap(short, long)]
  //  decompress: bool,
}

const BUFFER_SIZE : usize = 200*1024;

/**
 * 
 */
fn main()  {
    let args=Config::parse();
    print!("Heatshrinking ");
//    if args.decompress
//    {
//        print!("decompress ");
//    }else {
//        print!("compress ");
//    }
    println!(" {} => {} ", args.input_file,args.output_file);
    
    let mut buffer_in : [u8;BUFFER_SIZE] = [0;BUFFER_SIZE];
    let mut buffer_out : [u8;BUFFER_SIZE] = [0;BUFFER_SIZE];


    // 1- READ
    let mut ifile: File =  match  File::open(args.input_file.clone())
    {
        Ok(x) => x ,
        Err(_y) => {println!("Cannot open input file <{}>",args.input_file);panic!("!");},
    };
    let input_size = ifile.read(&mut buffer_in).expect("Cannot read intput file");
    drop(ifile); // make sure it's closed

    // 2- Compress/Decompress
    let cfg = hsConfig::new(7, 4).unwrap();
    let out : &[u8];
    /*
    if args.decompress{
        // use byte per byte decode
        let mut dec_size : usize = 0;
        let mut decoder =  hsdecoder::HeatshrinkDecoder::new(&buffer_in[..input_size], &cfg);
        
        for _i in 0..input_size
        {
            buffer_out[dec_size] = decoder.next();
            dec_size+=1;
        }
        out =&buffer_out[..dec_size];
    }else {*/
        out = encoder::encode(&buffer_in[..input_size], &mut buffer_out, &cfg).unwrap();
    //}

    // 3- Write

    let mut ofile: File =  match  File::create(args.output_file.clone())
    {
        Ok(x) => x ,
        Err(_y) => {println!("Cannot open output file <{}>",args.output_file);panic!("!");},
    };
    let output_size = out.len();
    ofile.write(out).expect("Error writing output file");
    
    drop(ofile); // make sure it's closed

    println!(" {} -> {}",input_size,output_size);

    println!("-Done-") 
    
}
//--eof--
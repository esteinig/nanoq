use clap::{Arg, ArgMatches, App};
use fastq::{parse_path, Record};

use std::str;
use libm::log10;

fn command_line_interface<'a>() -> ArgMatches<'a> {

    //Sets the command line interface of the program.

    App::new("nanoq")
            .version("0.0.1")
            .about("\nMinimal quality control for nanopore reads\n")
            .arg(Arg::from_usage("-f, --fastq=[FILE] 'Input read file or stream [-]'"))
            .arg(Arg::from_usage("-l, --length=[INT] 'Minimum read length [0]'"))
            .arg(Arg::from_usage("-q, --quality=[INT] 'Minimum read quality [0]'"))
            .get_matches()
}

fn main() {
    
    let cli = command_line_interface();

    let filename = match cli.value_of("fastq") {
        None | Some("-") => { None },
        Some(name) => Some(name)
    };

    let min_length: u64 = cli.value_of("length").unwrap_or("0").parse().unwrap();
    let min_quality: f64 = cli.value_of("quality").unwrap_or("0").parse().unwrap();

    parse_path(filename, |parser| {
                
        parser.each( |record| {
            
            let seq = match str::from_utf8(record.seq()) {
                Ok(v) => v, Err(e) => panic!("Invalid UTF-8 sequence: {}", e)
            };

            let head = match str::from_utf8(record.head()) {
                Ok(v) => v, Err(e) => panic!("Invalid UTF-8 sequence: {}", e)
            };

            let qual = match str::from_utf8(record.qual()) {
                Ok(v) => v, Err(e) => panic!("Invalid UTF-8 sequence: {}", e)
            };

            let seq_len = seq.len() as u64;
            
            // Nanopore quality score computation

            let error_probabilities: Vec<f64> = qual.chars().map(|x| get_error_probability(x)).collect();
            let mean_error: f64 = mean(&error_probabilities);
            let mean_quality: f64 = -10f64*log10(mean_error);

            if seq_len >= min_length && mean_quality >= min_quality{
                println!("@{}\n{}\n+\n{}", head, seq, qual);
            }            

            true

        }).expect("Invalid fastq file");
        
    }).expect("Invalid compression");
    
}

// Helper functions

fn get_error_probability(ascii: char) -> f64 {

    /* Compute the error probability from the quality score of a single base

    Quality encoding: Sanger Phred+33 --> ASCII: 36 - 126 --> Q: 0 - 93

    Computation of the base quality scores is described at:

    https://community.nanoporetech.com/technical_documents/data-analysis/

    https://gigabaseorgigabyte.wordpress.com/2017/06/26/averaging-basecall-quality-scores-the-right-way/

    */

    let sanger_phred = "!\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";
    
    match sanger_phred.find(ascii) {
        Some(v) => {
            10f64.powf(v as f64 / -10f64)
        }, 
        None => panic!("Invalid quality encoding: {}", ascii)
    }

}

fn mean(errors: &Vec<f64>) -> f64 {

    // Compute the mean of a vector of double-precision floats

    let sum: f64 = errors.iter().sum();

    sum as f64 / errors.len() as f64

}


use crate::cli::Cli;
use crate::cli::CompressionExt;
use std::fs::File;
use std::io::{BufWriter, Write};
use needletail::{FastxReader, parse_fastx_file, parse_fastx_stdin};
use thiserror::Error;
use std::io::{stdout, sink};

// Niffler output compression adopted from Michael B. Hall - Rasusa (https://github.com/mbhall88/rasusa)


/// A collection of custom errors relating to the working with fastx files.
#[derive(Error, Debug)]
pub enum FastxError {

    /// Indicates that a sequence record could not be parsed.
    #[error("Failed to parse record")]
    ParseError {
        source: needletail::errors::ParseError,
    },

}

pub struct NeedleCast {
    reader: Box<dyn FastxReader>,
    writer: Box<dyn Write>,

}

impl NeedleCast {

    pub fn new(args: &Cli) -> Self {

        let reader = match &args.input {
            Some(file) => parse_fastx_file(file).expect("failed to get file reader"),
            None => parse_fastx_stdin().expect("failed to get stdin reader"),
        };

        let writer = match &args.output {
            None => {
                if args.stats {
                    Box::new(sink())
                } else {
                    match args.output_type {
                        None => Box::new(stdout()),
                        Some(fmt) => niffler::basic::get_writer(
                            Box::new(stdout()), fmt, args.compress_level
                        ).expect("failed to get compressed stdout writer"),
                    }
                }
            },
            Some(output) => {

                let file = File::create(&output).expect("failed to create output file");
                let file_handle = Box::new(BufWriter::new(file));

                let fmt = match args.output_type {
                    None => niffler::Format::from_path(&output),
                    Some(f) => f,
                };

                niffler::get_writer(
                    file_handle, fmt, args.compress_level
                ).expect("failed to get compressed file writer")
            }
        };

        NeedleCast {
            reader, 
            writer
        }
    
    }
    pub fn filter(&mut self, min_length: u32, max_length: u32, min_quality: f32) -> Result<(Vec<u32>, Vec<f32>), FastxError> {
        let mut read_lengths: Vec<u32> = vec![];
        let mut read_qualities: Vec<f32> = vec![];

        
        let max_length: u32 = if max_length <= 0 { u32::MAX } else { max_length };

        while let Some(record) = self.reader.next() {
            match record {
                Ok(rec) => {
                    let seqlen = rec.num_bases() as u32;  // NANOQ READ LENGTH LIMIT: ~ 4.2 x 10e9
                    // Quality scores present (FASTQ not FASTA)
                    if let Some(qual) = rec.qual() {
                        let mean_error_prob = mean_error_probability(&qual);
                        let mean_quality: f32 = -10f32*mean_error_prob.log(10.0);
                        // FASTQ filter
                        if seqlen >= min_length && mean_quality >= min_quality && seqlen <= max_length {
                            read_lengths.push(seqlen);
                            read_qualities.push(mean_quality);
                            rec.write(&mut self.writer, None).expect("failed to write fastq record");
                        }
                    } else {
                        // FASTA filter
                        if seqlen >= min_length && seqlen <= max_length {
                            read_lengths.push(seqlen);
                            rec.write(&mut self.writer, None).expect("failed to write fasta record");
                        }
                    }

                },
                Err(err) => return Err(FastxError::ParseError { source: err }),
            }
        }
        Ok((read_lengths, read_qualities))
    }

    pub fn filter_length(&mut self, min_length: u32, max_length: u32) -> Result<(Vec<u32>, Vec<f32>), FastxError> {
        let mut read_lengths: Vec<u32> = vec![];
        let read_qualities: Vec<f32> = vec![];
        let max_length: u32 = if max_length <= 0 { u32::MAX } else { max_length };

        while let Some(record) = self.reader.next() {
            match record {
                Ok(rec) => {
                    let seqlen = rec.num_bases() as u32;
                    if seqlen >= min_length && seqlen <= max_length {
                        read_lengths.push(seqlen);
                        rec.write(&mut self.writer, None).expect("failed to write fastq record");
                    }

                },
                Err(err) => return Err(FastxError::ParseError { source: err }),
            }
        }
        Ok((read_lengths, read_qualities))
    }

}

/// Utility function to compute mean quality error for nanopore reads from bytes.
///
/// Quality encoding: Sanger Phred+33 --> ASCII: 33 - 126 --> Q: 0 - 93
///    f32 vs f64 makes a huge difference --> CHECK IF THIS WILL LIMIT READ LENGTH
///
/// Computation of the base quality scores is described at:
///
/// https://community.nanoporetech.com/technical_documents/data-analysis/
/// https://gigabaseorgigabyte.wordpress.com/2017/06/26/averaging-basecall-quality-scores-the-right-way/
///
fn mean_error_probability(quality_bytes: &[u8]) -> f32 {
    let mut sum: f32 = 0.0;
    for q in quality_bytes.iter(){
        sum += 10f32.powf((q-33u8) as f32 / -10f32)  // Q score and error probability
    }
    sum / quality_bytes.len() as f32  // Mean error probability
}
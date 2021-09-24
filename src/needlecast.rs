


use needletail::{FastxReader, parse_fastx_file, parse_fastx_stdin};
use std::io::{BufWriter, Write};
use std::io::{stdout, sink};
use thiserror::Error;
use std::fs::File;

use crate::cli::Cli;
use crate::cli::CompressionExt;


// Niffler output compression adopted from Michael B. Hall - Rasusa (https://github.com/mbhall88/rasusa)

/// A collection of custom errors relating to the working with fastx files.
#[derive(Error, Debug)]
pub enum FastxError {

    /// Indicates that a sequence record could not be parsed.
    #[error("Failed to parse sequence record")]
    ParseError {
        source: needletail::errors::ParseError,
    },

}

/// NeedleCast object
///
/// Basically a minimal wrapper around Needletail
/// that implements parsing and filtering based on
/// read length and quality
pub struct NeedleCast {
    reader: Box<dyn FastxReader>,
    writer: Box<dyn Write>,
}

impl NeedleCast {
    /// Create a new NeedleCast instance
    ///
    /// Given the command line interface object,
    /// create a new instance that parses its
    /// arguments and instantiates the
    /// reader and writer
    ///
    /// # Example
    ///
    /// ```rust
    /// let args = Cli::from_iter(&["nanoq"]);
    /// let nc = NeedleCast::new(&args);
    /// ```
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
    /// Filter reads and store lengths and qualities
    ///
    /// Given filtering parameters, iterate over reads
    /// and compute average read quality (fastq) records.
    /// Read lengths and qualities are stored in vectors
    /// and returned if no errors are raised
    /// 
    /// # Errors
    /// 
    /// If the sequence record cannot be parsed a variant
    /// of the `Needletail` `FastxError` is returned 
    ///
    /// # Example
    ///
    /// ```rust
    /// let args = Cli::from_iter(&["nanoq"]);
    /// let nc = NeedleCast::new(&args);
    /// nc.filter(0, 0, 0);
    /// ```
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
    /// Filter reads and store lengths and qualities
    /// without considering quality scores
    ///
    /// Given filtering parameters, iterate over reads
    /// but do not compute quality scores to speed up
    /// read iteration.
    /// 
    /// Read lengths and qualities are stored in vectors
    /// and returned if no errors are raised.
    /// 
    /// # Errors
    /// 
    /// If the sequence record cannot be parsed a variant
    /// of the `Needletail` `FastxError` is returned 
    ///
    /// # Example
    ///
    /// ```rust
    /// let args = Cli::from_iter(&["nanoq"]);
    /// let nc = NeedleCast::new(&args);
    /// nc.filter_length(0, 0, 0);
    /// ```
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

/// Utility function to compute mean error probability from quality bytes
///
/// This function computes the mean error probability from quality bytes,
/// from which the mean read quality can be computed.
///
/// Quality encoding: Sanger Phred+33 --> ASCII: 33 - 126 --> Q: 0 - 93
///
/// Computation of nanopore quality scores is described at:
///
/// https://community.nanoporetech.com/technical_documents/data-analysis/
///
/// # Example
/// 
/// ```compile
/// while let Some(record) = reader.next() {
///     let seqrec = record.expect("invalid record");
///     let qual_bytes = record.qual().expect("invalid quality bytes");
///     let error_prob = mean_error_probability(&qual_bytes);
///     let mean_error = -10f32*error_prob.log(10.0);
/// }
/// ```
fn mean_error_probability(quality_bytes: &[u8]) -> f32 {
    let mut sum: f32 = 0.0;
    for q in quality_bytes.iter(){
        sum += 10f32.powf((q-33u8) as f32 / -10f32)  
    }
    sum / quality_bytes.len() as f32  // mean error probability
}
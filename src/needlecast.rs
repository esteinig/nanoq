use needletail::errors::ParseError;
use needletail::{parse_fastx_file, parse_fastx_stdin, FastxReader};
use std::fs::File;
use std::io::{sink, stdout};
use std::io::{BufWriter, Write};
use thiserror::Error;

use crate::cli::Cli;
use crate::utils::CompressionExt;

// Niffler output compression adopted from Michael B. Hall - Rasusa (https://github.com/mbhall88/rasusa)

/// A collection of custom errors relating to the Needlecast class.
#[derive(Error, Debug)]
pub enum NeedlecastError {
    /// Indicates error in parsing Needletail Fastx
    #[error("Could not parse fastx file or stdin")]
    ParseFastx(#[from] needletail::errors::ParseError),
    /// Indicates error in Niffler compression format
    #[error("Could not get compressed writer")]
    CompressionError(#[from] niffler::Error),
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
    /// ```compile
    /// let cli = nanoq::cli::Cli::from_iter(&["nanoq"]);
    /// let caster = nanoq::needlecast::NeedleCast::new(&args);
    /// ```
    #[cfg(not(tarpaulin_include))]
    pub fn new(cli: &Cli) -> Result<Self, NeedlecastError> {
        let reader = match &cli.input {
            Some(file) => parse_fastx_file(file)?,
            None => parse_fastx_stdin()?,
        };
        let writer = match &cli.output {
            None => {
                if cli.stats {
                    Box::new(sink())
                } else {
                    match cli.output_type {
                        None => Box::new(stdout()),
                        Some(fmt) => {
                            niffler::basic::get_writer(Box::new(stdout()), fmt, cli.compress_level)?
                        }
                    }
                }
            }
            Some(output) => {
                let file = File::create(&output).expect("failed to create output file");
                let file_handle = Box::new(BufWriter::new(file));

                let fmt = match cli.output_type {
                    None => niffler::Format::from_path(&output),
                    Some(f) => f,
                };
                niffler::get_writer(file_handle, fmt, cli.compress_level)?
            }
        };
        Ok(NeedleCast { reader, writer })
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
    /// If the sequence record cannot be parsed a
    /// `needletail::errors::ParseError` is returned
    ///
    /// # Example
    ///
    /// ```compile
    /// use structopt::StructOpt;
    ///
    /// let cli = Cli::from_iter(&["nanoq", "-i", "tests/cases/test.fq", "-o", "/dev/null"]);
    /// let mut caster = NeedleCast::new(&cli);
    /// let (read_lengths, read_quals) = caster.filter(0, 0, 0.0).unwrap();
    ///
    /// assert_eq!(read_lengths, vec![4]);
    /// assert_eq!(read_quals, vec![40.0]);
    /// ```
    pub fn filter(
        &mut self,
        min_length: u32,
        max_length: u32,
        min_quality: f32,
        max_quality: f32,
    ) -> Result<(Vec<u32>, Vec<f32>, u64), ParseError> {
        let mut read_lengths: Vec<u32> = vec![];
        let mut read_qualities: Vec<f32> = vec![];

        let max_length: u32 = if max_length == 0 {
            u32::MAX
        } else {
            max_length
        };

        let max_quality = if max_quality == 0. { 93. } else { max_quality };

        let mut filtered: u64 = 0;
        while let Some(record) = self.reader.next() {
            let rec = record.expect("failed to parse record");
            let seqlen = rec.num_bases() as u32; // NANOQ READ LENGTH LIMIT: ~ 4.2 x 10e9
            
            //  Quality scores present (FASTQ not FASTA)
            if let Some(qual) = rec.qual() {
                let mean_error_prob = mean_error_probability(qual);
                let mean_quality: f32 = -10f32 * mean_error_prob.log(10.0);
                // FASTQ
                if seqlen >= min_length
                    && seqlen <= max_length
                    && mean_quality >= min_quality
                    && mean_quality <= max_quality
                {
                    read_lengths.push(seqlen);
                    read_qualities.push(mean_quality);
                    rec.write(&mut self.writer, None)
                        .expect("failed to write fastq record");
                } else {
                    filtered += 1;
                }
            } else {
                // FASTA
                if seqlen >= min_length && seqlen <= max_length {
                    read_lengths.push(seqlen);
                    rec.write(&mut self.writer, None)
                        .expect("failed to write fasta record");
                } else {
                    filtered += 1;
                }
            }
        }
        Ok((read_lengths, read_qualities, filtered))
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
    /// If the sequence record cannot be parsed a
    /// `needletail::errors::ParseError` is returned
    ///
    /// # Example
    ///
    /// ```compile
    /// let cli = nanoq::cli::Cli::from_iter(&["nanoq"]);
    /// let caster = nanoq::needlecast::NeedleCast::new(&cli);
    /// caster.filter_length(0, 0);
    /// ```
    pub fn filter_length(
        &mut self,
        min_length: u32,
        max_length: u32,
    ) -> Result<(Vec<u32>, Vec<f32>, u64), ParseError> {
        let mut read_lengths: Vec<u32> = vec![];
        let read_qualities: Vec<f32> = vec![];

        let max_length: u32 = if max_length == 0 {
            u32::MAX
        } else {
            max_length
        };
        let mut filtered: u64 = 0;
        while let Some(record) = self.reader.next() {
            let rec = record.expect("failed to parse record");
            let seqlen = rec.num_bases() as u32;
            if seqlen >= min_length && seqlen <= max_length {
                read_lengths.push(seqlen);
                rec.write(&mut self.writer, None)
                    .expect("failed to write fastq record");
            } else {
                filtered += 1;
            }
        }
        Ok((read_lengths, read_qualities, filtered))
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
/// ```rust
/// use needletail::parser::{FastqReader, FastxReader};
/// let fastq = b"@id\nACGT\n+\nIIII";
///
/// let mut reader = FastqReader::new(&fastq[..]);
/// let record = reader.next().unwrap().unwrap();
/// let qual_bytes = record.qual().unwrap();
/// let error_prob = mean_error_probability(&qual_bytes);
/// let mean_qual = -10f32*error_prob.log(10.0);
///
/// assert_eq!(error_prob, 0.0001);
/// assert_eq!(mean_qual, 40.0);
/// ```
fn mean_error_probability(quality_bytes: &[u8]) -> f32 {
    let mut sum: f32 = 0.0;
    for q in quality_bytes.iter() {
        sum += 10f32.powf((q - 33u8) as f32 / -10f32)
    }
    sum / quality_bytes.len() as f32 // mean error probability
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))] // weirdly includes line from [should_panic] tests
mod tests {
    use super::*;

    #[test]
    fn mean_error_probablity_and_quality_score() {
        use float_eq::float_eq;
        use needletail::parser::{FastqReader, FastxReader};

        let fastq = b"@id\nACGT\n+\nIIII";

        let mut reader = FastqReader::new(&fastq[..]);
        let record = reader.next().unwrap().unwrap();
        let qual_bytes = record.qual().unwrap();

        let error_prob = mean_error_probability(qual_bytes);
        let mean_qual = -10f32 * error_prob.log(10.0);

        float_eq!(error_prob, 0.0001, abs <= f32::EPSILON);
        float_eq!(mean_qual, 40.0, abs <= f32::EPSILON);
    }

    #[test]
    fn needlecast_filter_fq_ok() {
        use structopt::StructOpt;

        let cli = Cli::from_iter(&["nanoq", "-i", "tests/cases/test_ok.fq", "-o", "/dev/null"]);
        let mut caster = NeedleCast::new(&cli).unwrap();
        let (read_lengths, read_quals, _) = caster.filter(0, 0, 0.0, 0.0).unwrap();

        assert_eq!(read_lengths, vec![4]);
        assert_eq!(read_quals, vec![40.0]);
    }

    #[test]
    fn needlecast_filter_max_fq_ok() {
        use structopt::StructOpt;

        let cli = Cli::from_iter(&["nanoq", "-i", "tests/cases/test_ok.fq", "-o", "/dev/null"]);
        let mut caster = NeedleCast::new(&cli).unwrap();
        let (read_lengths, read_quals, _) = caster.filter(0, 3, 0.0, 0.0).unwrap();

        let expected_length: Vec<u32> = vec![];
        let expected_quality: Vec<f32> = vec![];

        assert_eq!(read_lengths, expected_length);
        assert_eq!(read_quals, expected_quality);
    }

    #[test]
    fn needlecast_filter_length_fq_ok() {
        use structopt::StructOpt;

        let cli = Cli::from_iter(&["nanoq", "-i", "tests/cases/test_len.fq", "-o", "/dev/null"]);
        let mut caster = NeedleCast::new(&cli).unwrap();
        let (read_lengths, read_quals, _) = caster.filter_length(0, 0).unwrap();

        let expected_quality: Vec<f32> = vec![];

        assert_eq!(read_lengths, vec![4, 8]);
        assert_eq!(read_quals, expected_quality);
    }

    #[test]
    fn needlecast_filter_length_max_fq_ok() {
        use structopt::StructOpt;

        let cli = Cli::from_iter(&["nanoq", "-i", "tests/cases/test_len.fq", "-o", "/dev/null"]);

        let mut caster = NeedleCast::new(&cli).unwrap();
        let (read_lengths, read_quals, _) = caster.filter_length(0, 3).unwrap();

        let expected_length: Vec<u32> = vec![];
        let expected_quality: Vec<f32> = vec![];

        assert_eq!(read_lengths, expected_length);
        assert_eq!(read_quals, expected_quality);

        // NeedleCast struct has to be initiated again to reset filter length parameters
        let mut caster = NeedleCast::new(&cli).unwrap();
        let (read_lengths, read_quals, _) = caster.filter_length(0, 5).unwrap();

        assert_eq!(read_lengths, vec![4]);
        assert_eq!(read_quals, expected_quality);
    }

    #[test]
    fn needlecast_filter_length_min_fq_ok() {
        use structopt::StructOpt;

        let cli = Cli::from_iter(&["nanoq", "-i", "tests/cases/test_len.fq", "-o", "/dev/null"]);

        let mut caster = NeedleCast::new(&cli).unwrap();
        let (read_lengths, read_quals, _) = caster.filter_length(5, 0).unwrap();

        let expected_quality: Vec<f32> = vec![];

        assert_eq!(read_lengths, vec![8]);
        assert_eq!(read_quals, expected_quality);

        // NeedleCast struct has to be initiated again to reset filter length parameters
        let mut caster = NeedleCast::new(&cli).unwrap();
        let (read_lengths, read_quals, _) = caster.filter_length(4, 0).unwrap();

        assert_eq!(read_lengths, vec![4, 8]);
        assert_eq!(read_quals, expected_quality);
    }

    #[test]
    fn needlecast_filter_fa_ok() {
        use structopt::StructOpt;

        let cli = Cli::from_iter(&["nanoq", "-i", "tests/cases/test_ok.fa", "-o", "/dev/null"]);
        let mut caster = NeedleCast::new(&cli).unwrap();
        let (read_lengths, read_quals, _) = caster.filter(0, 0, 0.0, 0.0).unwrap();

        let expected_quality: Vec<f32> = vec![];

        assert_eq!(read_lengths, vec![4]);
        assert_eq!(read_quals, expected_quality);
    }

    #[test]
    fn needlecast_filter_length_fa_ok() {
        use structopt::StructOpt;

        let cli = Cli::from_iter(&["nanoq", "-i", "tests/cases/test_ok.fa", "-o", "/dev/null"]);
        let mut caster = NeedleCast::new(&cli).unwrap();
        let (read_lengths, read_quals, _) = caster.filter_length(0, 0).unwrap();

        let expected_quality: Vec<f32> = vec![];

        assert_eq!(read_lengths, vec![4]);
        assert_eq!(read_quals, expected_quality);
    }

    #[test]
    #[should_panic]
    fn needlecast_filter_fa_fmt_bad() {
        use structopt::StructOpt;

        let cli = Cli::from_iter(&["nanoq", "-i", "tests/cases/test_bad1.fa", "-o", "/dev/null"]);
        let mut caster = NeedleCast::new(&cli).unwrap();
        caster.filter(0, 0, 0.0, 0.0).unwrap();
    }

    #[test]
    #[should_panic]
    fn needlecast_filter_fq_fmt_bad() {
        use structopt::StructOpt;

        let cli = Cli::from_iter(&["nanoq", "-i", "tests/cases/test_bad1.fq", "-o", "/dev/null"]);
        let mut caster = NeedleCast::new(&cli).unwrap();
        caster.filter(0, 0, 0.0, 0.0).unwrap();
    }

    #[test]
    #[should_panic]
    fn needlecast_filter_fq_sep_bad() {
        use structopt::StructOpt;

        let cli = Cli::from_iter(&["nanoq", "-i", "tests/cases/test_bad2.fq", "-o", "/dev/null"]);
        let mut caster = NeedleCast::new(&cli).unwrap();
        caster.filter(0, 0, 0.0, 0.0).unwrap();
    }

    #[test]
    #[should_panic]
    fn needlecast_filter_length_fq_fmt_bad() {
        use structopt::StructOpt;

        let cli = Cli::from_iter(&["nanoq", "-i", "tests/cases/test_bad1.fq", "-o", "/dev/null"]);
        let mut caster = NeedleCast::new(&cli).unwrap();
        caster.filter_length(0, 0).unwrap();
    }

    #[test]
    #[should_panic]
    fn needlecast_filter_length_fq_sep_bad() {
        use structopt::StructOpt;

        let cli = Cli::from_iter(&["nanoq", "-i", "tests/cases/test_bad2.fq", "-o", "/dev/null"]);
        let mut caster = NeedleCast::new(&cli).unwrap();
        caster.filter_length(0, 0).unwrap();
    }
}

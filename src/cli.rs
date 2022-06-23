use std::path::PathBuf;
use structopt::StructOpt;
use thiserror::Error;

/// Read filters and summary reports for nanopore data
#[derive(Debug, StructOpt)]
#[structopt()]
pub struct Cli {
    /// Fast{a,q}.{gz,xz,bz}, stdin if not present.
    #[structopt(short, long, parse(from_os_str))]
    pub input: Option<PathBuf>,

    /// Output filepath, stdout if not present.
    #[structopt(short, long, parse(from_os_str))]
    pub output: Option<PathBuf>,

    /// Minimum read length filter (bp).
    #[structopt(short = "l", long, value_name = "INT", default_value = "0")]
    pub min_len: u32,

    /// Maximum read length filter (bp).
    #[structopt(short = "m", long, value_name = "INT", default_value = "0")]
    pub max_len: u32,

    /// Minimum average read quality filter (Q).
    #[structopt(short = "q", long, value_name = "FLOAT", default_value = "0")]
    pub min_qual: f32,

    /// Maximum average read quality filter (Q).
    #[structopt(short = "w", long, value_name = "FLOAT", default_value = "0")]
    pub max_qual: f32,

    /// Verbose output statistics [multiple, up to -vvv]  
    #[structopt(
        short,
        long,
        parse(from_occurrences = parse_verbosity)
    )]
    pub verbose: u64,

    /// Header for summary output
    #[structopt(short = "H", long)]
    pub header: bool,

    /// Number of top reads in verbose summary.  
    #[structopt(short, long, value_name = "INT", default_value = "5")]
    pub top: usize,

    /// Summary report only [stdout].
    #[structopt(short, long)]
    pub stats: bool,

    /// Summary report output file.
    #[structopt(short, long)]
    pub report: Option<PathBuf>,

    /// Summary report in JSON format.
    #[structopt(short, long)]
    pub json: bool,

    /// Read lengths output file.
    #[structopt(short, long)]
    pub read_lengths: Option<PathBuf>,

    /// Read qualities output file.
    #[structopt(short, long)]
    pub read_qualities: Option<PathBuf>,

    /// Ignore quality values if present.
    #[structopt(short, long)]
    pub fast: bool,

    /// u: uncompressed; b: Bzip2; g: Gzip; l: Lzma
    ///
    /// Nanoq will attempt to infer the output compression format automatically
    /// from the filename extension. This option is used to override that.
    /// If writing to stdout, the default is uncompressed
    #[structopt(
        short = "O", 
        long,
        value_name = "u|b|g|l", 
        parse(try_from_str = parse_compression_format),
        possible_values = &["u", "b", "g", "l"], 
        case_insensitive = true,
        hide_possible_values = true
    )]
    pub output_type: Option<niffler::compression::Format>,

    /// Compression level to use if compressing output.
    #[structopt(
        short = "c", 
        long,
        parse(try_from_str = parse_compression_level),
        default_value="6", 
        value_name = "1-9"
    )]
    pub compress_level: niffler::Level,
}

/// A collection of custom errors relating to the command line interface for this package.
#[derive(Error, Debug, PartialEq)]
pub enum CliError {
    /// Indicates that a string cannot be parsed into a [`CompressionFormat`](#compressionformat).
    #[error("{0} is not a valid output format")]
    InvalidCompressionFormat(String),

    /// Indicates that a string cannot be parsed into a [`CompressionLevel`](#compressionlevel).
    #[error("{0} is not a valid compression level [1-9]")]
    InvalidCompressionLevel(String),
}

/// Utility function to parse verbosity occurences
///
/// Up to three verbosity flags are allowed (-vvv), if more
/// are specified (-vvvv) the highest allowed value is returned
pub fn parse_verbosity(v: u64) -> u64 {
    match v {
        0 | 1 | 2 | 3 => v,
        _ => 3,
    }
}

/// Utility function to parse compression format
fn parse_compression_format(s: &str) -> Result<niffler::compression::Format, CliError> {
    match s {
        "b" | "B" => Ok(niffler::Format::Bzip),
        "g" | "G" => Ok(niffler::Format::Gzip),
        "l" | "L" => Ok(niffler::Format::Lzma),
        "u" | "U" => Ok(niffler::Format::No),
        _ => Err(CliError::InvalidCompressionFormat(s.to_string())),
    }
}

/// Utility function to parse and validate compression level
#[allow(clippy::redundant_clone)]
fn parse_compression_level(s: &str) -> Result<niffler::Level, CliError> {
    let lvl = match s.parse::<u8>() {
        Ok(1) => niffler::Level::One,
        Ok(2) => niffler::Level::Two,
        Ok(3) => niffler::Level::Three,
        Ok(4) => niffler::Level::Four,
        Ok(5) => niffler::Level::Five,
        Ok(6) => niffler::Level::Six,
        Ok(7) => niffler::Level::Seven,
        Ok(8) => niffler::Level::Eight,
        Ok(9) => niffler::Level::Nine,
        _ => return Err(CliError::InvalidCompressionLevel(s.to_string())),
    };
    Ok(lvl)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_compression_format() {
        let passed_args = vec!["nanoq", "-O", "t"];
        let args: Result<Cli, clap::Error> = Cli::from_iter_safe(passed_args);

        let actual = args.unwrap_err().kind;
        let expected = clap::ErrorKind::InvalidValue;

        assert_eq!(actual, expected)
    }

    #[test]
    fn invalid_compression_level() {
        let passed_args = vec!["nanoq", "-c", "10"];
        let args: Result<Cli, clap::Error> = Cli::from_iter_safe(passed_args);

        let actual = args.unwrap_err().kind;
        let expected = clap::ErrorKind::ValueValidation;

        assert_eq!(actual, expected)
    }

    #[test]
    fn verbosity_exceeds_limit() {
        let passed_args = vec!["nanoq", "-vvvv"];
        let args = Cli::from_iter_safe(passed_args);

        let actual = args.unwrap().verbose;
        let expected = 3;

        assert_eq!(actual, expected)
    }

    #[test]
    fn invalid_min_len() {
        let passed_args = vec!["nanoq", "-l", "test"];
        let args: Result<Cli, clap::Error> = Cli::from_iter_safe(passed_args);

        let actual = args.unwrap_err().kind;
        let expected = clap::ErrorKind::ValueValidation;

        assert_eq!(actual, expected)
    }

    #[test]
    fn invalid_max_len() {
        let passed_args = vec!["nanoq", "-m", "test"];
        let args: Result<Cli, clap::Error> = Cli::from_iter_safe(passed_args);

        let actual = args.unwrap_err().kind;
        let expected = clap::ErrorKind::ValueValidation;

        assert_eq!(actual, expected)
    }

    #[test]
    fn invalid_min_qual() {
        let passed_args = vec!["nanoq", "-q", "test"];
        let args: Result<Cli, clap::Error> = Cli::from_iter_safe(passed_args);

        let actual = args.unwrap_err().kind;
        let expected = clap::ErrorKind::ValueValidation;

        assert_eq!(actual, expected)
    }

    #[test]
    fn invalid_max_qual() {
        let passed_args = vec!["nanoq", "-w", "test"];
        let args: Result<Cli, clap::Error> = Cli::from_iter_safe(passed_args);

        let actual = args.unwrap_err().kind;
        let expected = clap::ErrorKind::ValueValidation;

        assert_eq!(actual, expected)
    }

    #[test]
    fn invalid_to_value() {
        let passed_args = vec!["nanoq", "-t", "test"];
        let args: Result<Cli, clap::Error> = Cli::from_iter_safe(passed_args);

        let actual = args.unwrap_err().kind;
        let expected = clap::ErrorKind::ValueValidation;

        assert_eq!(actual, expected)
    }

    #[test]
    fn valid_stats_flag() {
        let passed_args = vec!["nanoq", "-s"];
        let args: Result<Cli, clap::Error> = Cli::from_iter_safe(passed_args);

        let actual = args.unwrap().stats;
        let expected = true;

        assert_eq!(actual, expected)
    }

    #[test]
    fn valid_fast_flag() {
        let passed_args = vec!["nanoq", "-f"];
        let args: Result<Cli, clap::Error> = Cli::from_iter_safe(passed_args);

        let actual = args.unwrap().fast;
        let expected = true;

        assert_eq!(actual, expected)
    }

    #[test]
    fn valid_verbosity_level() {
        let passed_args = vec!["nanoq", "-vv"];
        let args = Cli::from_iter_safe(passed_args);

        let actual = args.unwrap().verbose;
        let expected = 2;

        assert_eq!(actual, expected)
    }

    #[test]
    fn verbosity_from_occurrences() {
        assert_eq!(parse_verbosity(0), 0);
        assert_eq!(parse_verbosity(1), 1);
        assert_eq!(parse_verbosity(2), 2);
        assert_eq!(parse_verbosity(3), 3);
        assert_eq!(parse_verbosity(4), 3);
        assert_eq!(parse_verbosity(666), 3);
    }

    #[test]
    fn compression_format_from_str() {
        let mut s = "B";
        assert_eq!(parse_compression_format(s).unwrap(), niffler::Format::Bzip);

        s = "g";
        assert_eq!(parse_compression_format(s).unwrap(), niffler::Format::Gzip);

        s = "l";
        assert_eq!(parse_compression_format(s).unwrap(), niffler::Format::Lzma);

        s = "U";
        assert_eq!(parse_compression_format(s).unwrap(), niffler::Format::No);

        s = "a";
        assert_eq!(
            parse_compression_format(s).unwrap_err(),
            CliError::InvalidCompressionFormat(s.to_string())
        );
    }

    #[test]
    fn compression_level_in_range() {
        assert!(parse_compression_level("1").is_ok());
        assert!(parse_compression_level("2").is_ok());
        assert!(parse_compression_level("3").is_ok());
        assert!(parse_compression_level("4").is_ok());
        assert!(parse_compression_level("5").is_ok());
        assert!(parse_compression_level("6").is_ok());
        assert!(parse_compression_level("7").is_ok());
        assert!(parse_compression_level("8").is_ok());
        assert!(parse_compression_level("9").is_ok());
        assert!(parse_compression_level("0").is_err());
        assert!(parse_compression_level("10").is_err());
        assert!(parse_compression_level("f").is_err());
        assert!(parse_compression_level("5.5").is_err());
        assert!(parse_compression_level("-3").is_err());
    }
}

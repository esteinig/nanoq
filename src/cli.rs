use std::path::{Path, PathBuf};
use structopt::StructOpt;
use thiserror::Error;
use std::ffi::OsStr;

/// Read filters and summary reports for nanopore data
#[derive(Debug, StructOpt)]
#[structopt()]
pub struct Cli {
    /// Fast{a,q}.{gz,xz,bz}, stdin if not present.
    #[structopt(
        short,
        long,
        parse(from_os_str)
    )]
    pub input: Option<PathBuf>,

    /// Output filepath, stdout if not present.
    #[structopt(
        short, 
        long, 
        parse(from_os_str)
    )]
    pub output: Option<PathBuf>,

    /// Minimum read length filter (bp).
    #[structopt(
        short = "l",
        long,
        value_name = "INT", 
        default_value = "0"
    )]
    pub min_len: u32,

    /// Maximum read length filter (bp).
    #[structopt(
        short = "m",
        long,
        value_name = "INT", 
        default_value = "0" 
    )]
    pub max_len: u32,

    /// Minimum average read quality filter (Q).
    #[structopt(
        short,
        long,
        value_name = "FLOAT", 
        default_value = "0" 
    )]
    pub min_qual: f32,

    /// Pretty print output statistics.  
    #[structopt(
        short,
        long,
        parse(from_occurrences)
    )]
    pub verbose: u8,

    /// Number of top reads in verbose summary.  
    #[structopt(
        short,
        long,
        value_name = "INT", 
        default_value = "5" 
    )]
    pub top: usize,

    /// Statistics only, reads to /dev/null.
    #[structopt(
        short,
        long
    )]
    pub stats: bool,

    /// Ignore quality values if present.
    #[structopt(
        short,
        long
    )]
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


// Adopted from Michael B. Hall - Rasusa (https://github.com/mbhall88/rasusa)

pub trait CompressionExt {
    fn from_path<S: AsRef<OsStr> + ?Sized>(p: &S) -> Self;
}

/// Attempts to infer the compression type from the file extension. 
/// If the extension is not known, then Uncompressed is returned.
impl CompressionExt for niffler::compression::Format {
    fn from_path<S: AsRef<OsStr> + ?Sized>(p: &S) -> Self {
        let path = Path::new(p);
        match path.extension().map(|s| s.to_str()) {
            Some(Some("gz")) => Self::Gzip,
            Some(Some("bz") | Some("bz2")) => Self::Bzip,
            Some(Some("lzma")) => Self::Lzma,
            _ => Self::No,
        }
    }
}


/// Utility function to parse compression format raising error if no valid format is provided
fn parse_compression_format(s: &str) -> Result<niffler::compression::Format, CliError> {
    match s {
        "b" | "B" => Ok(niffler::Format::Bzip),
        "g" | "G" => Ok(niffler::Format::Gzip),
        "l" | "L" => Ok(niffler::Format::Lzma),
        "u" | "U" => Ok(niffler::Format::No),
        _ => Err(CliError::InvalidCompressionFormat(s.to_string())),
    }
}

/// Utility function to validate compression level is in allowed range
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
        _ => return Err(CliError::InvalidCompressionLevel(s.to_string()))
    };
    Ok(lvl)
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn no_input_file_given_raises_error() {
        let passed_args = vec!["nanoq", "-l", "1000"];
        let args: Result<Cli, clap::Error> = Cli::from_iter_safe(passed_args);

        let actual = args.unwrap_err().kind;
        let expected = clap::ErrorKind::MissingRequiredArgument;

        assert_eq!(actual, expected)
    }


}
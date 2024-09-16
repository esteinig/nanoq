use std::path::PathBuf;
use clap::{crate_version, Args, Parser, Subcommand};

use crate::prelude::*;

#[derive(Debug, Parser)]
#[command(
    author="Eike Steinig (@esteinig)", 
    version=crate_version!(), 
    about="Minimal but speedy quality control and summaries of nanopore reads",
    help_template="\
{before-help}{name} {version} 
{author-with-newline}
{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}
"
)]
#[command(styles=get_styles())]
#[command(arg_required_else_help(true))]
pub struct App {
    /// Output logs to file instead of terminal
    ///
    /// Specify a file path to store the output logs. If not provided,
    /// logs will be displayed in the terminal. 
    #[arg(short, long)]
    pub log: Option<PathBuf>,
    /// Increase output verbosity
    ///
    /// Use multiple `-v` or `--verbose` flags to increase verbosity level.
    /// For example, `-v` for info, `-vv` for debug, `-vvv` for trace.
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
    /// Nanoq quality control commands
    #[clap(subcommand)]
    pub command: Commands,
}

/// Enumeration of available commands for Scrubby.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Quality control for long reads
    Reads(ReadArgs),
}

/// Command-line arguments for the cleaning operation
///
/// This struct defines the arguments required for performing the cleaning 
/// operation using Scrubby. It includes options for specifying input and output 
/// files, aligners, classifiers, and various other parameters.
#[derive(Args, Debug)]
pub struct ReadArgs {
    /// Input reads (.fastq) or sequences (.fasta) (.gz | .xz | .bz)
    ///
    /// One or two input read or sequencefiles. This parameter is required and multiple file
    /// can be specified either consecutively or using multiple input arguments, for example: 
    /// '-i part1.fq.gz -i part2.fq.gz' or '-i part1.fq.gz part2.fq.gz'. 
    #[arg(short, long, num_args(0..))]
    input: Vec<PathBuf>,
    /// Filtered read output (.gz | .xz | .bz)
    ///
    /// Filtered read output. This parameter is optional and a single
    /// file can be specified. Defaults to 'stdout'.
    #[arg(short, long)]
    output: Option<PathBuf>,
    /// Number of threads to use for aligner and classifier
    ///
    /// Number of threads to be used by the aligner and classifier. 
    /// The default value is 4.
    #[arg(short, long, default_value = "4")]
    threads: usize,
    /// Summary output file (.json)
    ///
    /// Path to a JSON file for the output summary.
    #[arg(short, long)]
    json: Option<PathBuf>,
}
impl ReadArgs {
    /// Validates the provided arguments and builds a 'Nanoq' instance.
    ///
    /// This method checks the provided arguments for consistency and constructs 
    /// a `Nanoq` instance based on the validated arguments.
    ///
    /// # Returns
    ///
    /// * `Result<Nanoq, Error>` - Ok with the constructed Nanoq instance, otherwise an error.
    /// 
    /// # Example
    ///
    /// ```
    /// use clap::Parser;
    /// 
    /// let reads_Args = ReadsArgs::parse();
    /// let scrubby = reads_Args.validate_and_build().unwrap();
    /// ```
    pub fn validate_and_build(self) -> Result<Nanoq, Error> {

        let command = std::env::args().collect::<Vec<String>>().join(" ");
        
        let builder = NanoqBuilder::new(self.input, self.output)
            .command(command)
            .json(self.json)
            .threads(self.threads);

        let nanoq = builder.build()?;

        Ok(nanoq)
    }
}


/// Configures the styles for the command-line interface.
pub fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .usage(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
        )
        .header(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
        )
        .literal(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green))),
        )
}

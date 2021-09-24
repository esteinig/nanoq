

use anyhow::{Context, Result};
use structopt::StructOpt;

use crate::cli::Cli;
use crate::utils::ReadSet;
use crate::needlecast::NeedleCast;

mod cli;
mod needlecast;
mod utils;

/// Nanoq application
///
/// Run the application from arguments provided 
/// by the command line interface.
fn main() -> Result<()> {
    let args: Cli = Cli::from_args();
    let mut needle_cast = NeedleCast::new(&args);

    let (read_lengths, read_qualities) = match args.fast {
        true =>  needle_cast
                    .filter_length(args.min_len, args.max_len)
                    .context("unable to process reads")?,
        false => needle_cast
                    .filter(args.min_len, args.max_len, args.min_qual)
                    .context("unable to process reads")?
    };
    
    let mut read_set = ReadSet {
        read_lengths, read_qualities,
    };
    read_set
        .summary(&args.verbose, args.top)
        .context("unable to obtain read summary")?;

    Ok(())
}
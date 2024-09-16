use anyhow::{Context, Result};
use structopt::StructOpt;

use crate::cli::Cli;
use crate::needlecast::NeedleCast;
use crate::utils::ReadSet;

mod cli;
mod needlecast;
mod utils;

/// Nanoq application
///
/// Run the application from arguments provided
/// by the command line interface.
#[cfg(not(tarpaulin_include))]
fn main() -> Result<()> {
    let cli: Cli = Cli::from_args();
    let mut needle_cast = NeedleCast::new(&cli)?;

    let (read_lengths, read_qualities, n_filtered) = match cli.fast {
        true => needle_cast
            .filter_length(cli.min_len, cli.max_len, cli.trim_start, cli.trim_end)
            .context("unable to process reads")?,
        false => needle_cast
            .filter(
                cli.min_len,
                cli.max_len,
                cli.min_qual,
                cli.max_qual,
                cli.trim_start,
                cli.trim_end,
            )
            .context("unable to process reads")?,
    };

    let mut read_set = ReadSet::new(read_lengths, read_qualities);

    let output_data = read_set.get_output_data(cli.top, n_filtered);

    read_set
        .summary(
            output_data,
            &cli.verbose,
            cli.header,
            cli.stats,
            cli.json,
            cli.report,
        )
        .context("unable to get summary")?;

    if let Some(path) = cli.read_lengths {
        read_set.write_read_lengths(path)?;
    }
    if let Some(path) = cli.read_qualities {
        read_set.write_read_qualities(path)?;
    }
    Ok(())
}

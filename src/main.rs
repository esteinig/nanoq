use clap::Parser;
use nanoq::utils::init_logger;
use nanoq::terminal::{App, Commands};


fn main() -> anyhow::Result<()> {
    
    let cli = App::parse();

    init_logger(cli.log_file);

    match cli.command {
        Commands::Reads(args) => {
            args.validate_and_build()?.run()?;
        },
    }

    Ok(())
}
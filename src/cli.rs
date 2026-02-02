use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, propagate_version=true)]
pub struct Cli{
    // Compress or Decompress subcommand
    #[command(subcommand)]
    subcom : SubComs
}

#[derive(Subcommand, Debug)]
enum SubComs {
    // Compress Subcommand
    Compress {
        path : String
    },
    // Decompress Subcommand
    Decompress {
        path : String
    }
}

pub fn cli_parse() -> Cli {
    Cli::parse()
}
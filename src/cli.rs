use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, propagate_version=true)]
pub struct Cli {
    // Compress or Decompress subcommand
    #[command(subcommand)]
    pub subcom: SubComs,
}

#[derive(Subcommand, Debug)]
pub enum SubComs {
    /// Compress the file given in argument
    Compress {
        path: String,

        /// large Window size for compression
        #[arg(short, long)]
        large: bool,

        /// Medium Window size for compression
        #[arg(short, long)]
        medium: bool,

        /// Small Window size for compression
        #[arg(short, long)]
        small: bool,
    },
    /// Decompress the file given in argument
    Decompress { path: String },
}

pub fn cli_parse() -> Cli {
    Cli::parse()
}

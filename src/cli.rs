use clap::{Parser, Subcommand, Args};
use std::path::Path;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version=true)]
pub struct Arg{
    // Select Compress or Decompress mode
    #[command(subcommand)]
    pub command : Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    // Compress a file
    Compress(CompArgs),

    // Decompress a file
    Decompress(DecompArgs)
}

#[derive(Args)]
pub struct CompArgs{
    // Path to the file to compress
    pub path : Path,
}

#[derive(Args)]
pub struct DecompArgs{
    // Path to the file to decompress
    pub path : Path,
}

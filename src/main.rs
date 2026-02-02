use anyhow::Result;
use std::fs;
use std::path::Path;

mod cli;
mod compress;
mod decompress;
mod fhandler;

use crate::{
    compress::SlidingWindow,
    decompress::decomp,
    fhandler::{generate_output, get_file_data},
};
use cli::*;

fn main() -> Result<()> {
    let cli = cli::cli_parse();
    match &cli.subcom {
        SubComs::Compress {
            path,
            large,
            medium,
            small,
        } => {
            let p = Path::new(path);
            let capacity = if *large {
                32000usize
            } else if *medium {
                16000usize
            } else if *small {
                8000usize
            } else {
                4000usize
            };
            let mut buff: Vec<u8> = Vec::new();
            get_file_data(p, &mut buff)?;
            let mut sw = SlidingWindow::new(capacity, buff);
            let (f, e) = generate_output(p, false, None)?.unwrap();
            sw.compress(&f, e)?;
            Ok(())
        }
        SubComs::Decompress { path } => {
            let p = Path::new(path);
            decomp(p)?;
            Ok(())
        }
    }
}

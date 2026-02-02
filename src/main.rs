use anyhow::Result;
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
                44000usize // 33ko for search buffer and 11ko for look ahead buffer
            } else if *medium {
                24000usize // 18ko for search buffer and 6ko for look ahead buffer
            } else if *small {
                16000usize // 12ko for search buffer and 4ko for look ahead buffer
            } else {
                24000usize
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

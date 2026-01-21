use anyhow::Result;
use std::path::Path;
use std::fs::File;
use core::str;
use crate::fhandler::generate_output;

fn create_o(content : &Vec<u8>, p_to_f : &Path) ->Result<usize> { /// Returns the byte that has been read just for og file extension extraction
    let mut i = 0usize;
    let mut buff : Vec<u8> = Vec::new();
    while content[i] != "\n".as_bytes()[0] { // Because original file extension is at the header of the compressed file, we only stop when we see a \n
        buff.push(content[i]);
        i+= 1;
    }
    // We decode the buff to use it as the output file extension
    let e = str::from_utf8(&buff)?.to_string();
    // We generate the output with this extension
    generate_output(p_to_f, true, Some(e))?;
    Ok(i+1usize)
}

fn extract(f_content : Vec<u8>, mut output : File, i : usize) -> Result<()>{ /// "i" parameter is the position after og file extension extraction
    Ok(())
}

pub fn decomp(path_to_file : &Path) -> Result<()>{
    Ok(())
}

#[cfg(test)]
mod test {
}
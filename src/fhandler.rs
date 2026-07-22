use anyhow::{Error, Result};
use std::fs;
use std::fs::File;
use std::path::Path;

pub fn get_file_data(p: &Path, buffer: &mut Vec<u8>) -> Result<()> {
    // Fill the buffer with file datas
    for byte in fs::read(p)? {
        buffer.push(byte);
    }
    Ok(())
}

pub fn generate_output(
    p: &Path,
    decomp: bool,
    og_extension: Option<String>,
) -> Result<Option<(File, String)>> {
    // Modify the path to create the output file
    // Recover path buffer from p
    let mut path_to_file = p.to_path_buf();
    // Set flags
    let p_is_file = path_to_file.is_file();
    let p_is_compressed = path_to_file.extension().unwrap().to_str().unwrap() == "lzss";

    if p_is_file {
        if decomp {
            // if we want to decompress the file
            if p_is_compressed {
                if let Some(ext) = og_extension {
                    // We verify and recover the original file extension
                    // We set output extension
                    path_to_file.set_extension(&ext);
                    //We create the output file
                    let output = std::fs::File::create(path_to_file)?;
                    Ok(Some((output, ext)))
                } else {
                    Err(Error::msg("Original Extension missing"))
                }
            } else {
                Err(Error::msg("file is not compressed in format lzss"))
            }
        } else {
            // if we want to compress the file
            if !p_is_compressed {
                // We have to recover the original file extension
                let extension = String::from(path_to_file.extension().unwrap().to_str().unwrap());
                // We have to create the output file with lzss extension
                path_to_file.set_extension("lzss");
                let output = std::fs::File::create(path_to_file)?;
                Ok(Some((output, extension)))
            } else {
                Err(Error::msg("File already compressed"))
            }
        }
    } else {
        Err(Error::msg("Entered path isn't a file"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{assert_eq, io::Write};
    use tempfile::Builder;

    //Helper
    fn create_tmp_file(name: &str, content: &str) -> tempfile::NamedTempFile {
        let mut f = Builder::new()
            .prefix(name)
            .suffix(".ran")
            .tempfile()
            .unwrap();
        write!(f, "{}", content).unwrap();
        f
    }

    #[test]
    fn test_generate_output() {
        // NORMAL COMPRESSION TEST
        // Create temporary file
        let tmp_file = create_tmp_file("test.tmp", "hello world");
        
        // Generate output
        let res_process = generate_output(tmp_file.path(), false, None);

        // Checking process
        assert_eq!(res_process.is_ok(), true);

        // Verifying Generation
        let res_generation = res_process.unwrap();
        assert_eq!(res_generation.is_some(), true);

        // NO FILE COMPRESSION TEST
        let rp2 = generate_output(Path::new("test.nofile"), false, None);

        // Checking process
        assert_eq!(rp2.is_err(), true);

        // NORMAL DECOMPRESS TEST
        let comp_tmp_file = tmp_file.path().with_extension("lzss");
        let rp3 = generate_output(comp_tmp_file.as_path(), true, Some("tmp".to_string()));

        // Checking process
        assert_eq!(rp3.is_ok(), true);

        // Verify Generation
        let rg3 = rp3.unwrap();
        assert_eq!(rg3.is_some(), true);

        // WITHOUT OG EXTENSION TEST
        let rp4 = generate_output(comp_tmp_file.as_path(), true, None);

        // Checking process
        assert_eq!(rp4.is_err(), true);

        // NO NEED DECOMPRESSION FILE TEST
        let rp5 = generate_output(tmp_file.path(), true, Some("tmp".to_string()));

        // Checking process
        assert_eq!(rp5.is_err(), true);

        // ALREADY COMPRESSED FILE TEST
        let rp6 = generate_output(comp_tmp_file.as_path(), false, None);

        // Checking process
        assert_eq!(rp6.is_err(), true);
    }

    #[test]
    fn test_get_file_data() {
        let binding = create_tmp_file("getfdata", "Testing the get file data function right now !");
        let path = binding.path();
        let mut content: Box<Vec<u8>> = Box::new(Vec::new());
        let res = get_file_data(path, &mut content);
        assert_eq!(res.is_ok(), true);
        assert_eq!(content.len() == 0, false);
    }
}

use anyhow::Result;
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

pub fn generate_output(p: &Path, decomp : bool, og_extension : Option<String>) -> Result<Option<(File, String)>> {
    // Modify the path to create the output file
    // Recover path buffer from p
    let mut path_to_file = p.to_path_buf();
    // Set flags
    let p_is_file = path_to_file.is_file();
    let p_is_compressed = path_to_file.extension().unwrap().to_str().unwrap() == "lzss";
    
    if p_is_file {
        if decomp { // if we want to decompress the file
            if p_is_compressed {
                if og_extension.is_some() {
                    // We recover the original file extension
                    let ext = og_extension.unwrap();
                    // We set output extension
                    path_to_file.set_extension(&ext);
                    //We create the output file
                    let output = std::fs::File::create(path_to_file)?;
                    Ok(Some((output, ext)))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        } else { // if we want to compress the file
            if !p_is_compressed {
                // We have to recover the original file extension
                let extension = String::from(path_to_file.extension().unwrap().to_str().unwrap());
                // We have to create the output file with lzss extension
                path_to_file.set_extension("lzss");
                let output = std::fs::File::create(path_to_file)?;
                Ok(Some((output, extension)))
            } else {
                Ok(None)
            }
        }
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
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
        let res1 = generate_output(Path::new("test.txt"), false, None);
        assert_eq!(res1.is_ok(), true);
        let res2 = res1.unwrap();
        assert_eq!(res2.is_some(), true);
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

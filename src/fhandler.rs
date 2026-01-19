use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::fs::File;

pub fn get_file_data(p:&Path, buffer : &mut Box<Vec<u8>>) -> Result<()>{
    // Recover the mutable buffer content
    let vec_ptr = Box::as_mut(buffer);
    for byte in fs::read(p)? {
        vec_ptr.push(byte);
    }
    Ok(())
}

pub fn generate_output(p : &Path) -> Result<File>{
    // Modify the path to create the output file
    // Set the path to the new file
    let mut path_to_file = p.to_path_buf();
    path_to_file.set_extension("lzss");
    // We have to create the output file with lzss extension
    let output = std::fs::File::create(path_to_file)?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::Builder;
    use std::io::Write;

    //Helper
    fn create_tmp_file(name : &str, content : &str) -> tempfile::NamedTempFile{
        let mut f = Builder::new().prefix(name).suffix(".ran").tempfile().unwrap();
        write!(f,"{}", content).unwrap();
        f
    }

    #[test]
    fn test_generate_output(){
        let output = generate_output(Path::new("test.txt"));
        assert_eq!(output.is_ok(), true);
        let file = output.unwrap();
        let metadata = file.metadata().unwrap();
    }

    #[test]
    fn test_get_file_data(){
        let binding = create_tmp_file("getfdata","Testing the get file data function right now !");
        let path = binding.path();
        let mut content : Box<Vec<u8>> = Box::new(Vec::new());
        let res = get_file_data(path, &mut content);
        assert_eq!(res.is_ok(), true);
        assert_eq!(content.len() == 0, false);
    }
}
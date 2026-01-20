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

pub fn generate_output(p: &Path) -> Result<File> {
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
        let output = generate_output(Path::new("test.txt"));
        assert_eq!(output.is_ok(), true);
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

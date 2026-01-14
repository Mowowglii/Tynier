use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn get_file_data(p:&Path, buffer : &mut Box<Vec<u8>>) -> Result<()>{
    // Recover the mutable buffer content
    let vec_ptr = Box::as_mut(buffer);
    for byte in fs::read(p)? {
        vec_ptr.push(byte);
    }
    Ok(())
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
    fn test_get_file_data(){
        let binding = create_tmp_file("getfdata","Testing the get file data function right now !");
        let path = binding.path();
        let mut content : Box<Vec<u8>> = Box::new(Vec::new());
        let res = get_file_data(path, &mut content);
        assert_eq!(res.is_ok(), true);
        assert_eq!(content.len() == 0, false);
    }
}
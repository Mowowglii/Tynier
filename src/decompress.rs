use anyhow::Result;
use std::path::Path;
use std::fs::File;
use core::str;
use crate::fhandler::generate_output;

struct DecompToken {
    offset : usize,
    length : usize
}

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
    Ok(i+1usize) // Because we only need to extract everything after the "\n"
}

fn gives_token(content : &Vec<u8>) -> Option<DecompToken>{ // Receiving in parameter content, the bytes after the first occurance of "<"
    // Searching the end of the token and the separator position
    let mut sep_position : usize = 0usize;
    let mut j = 0usize;
    let mut sep_flag = false; // We haven't encountered the separator yet

    while content[j] != 62u8 {
        if content[j] == 59u8 && sep_flag == false { // We found the position of the separator
            sep_position = j;
            sep_flag = true;
        } else if content[j] == 59u8 && sep_flag == true { // We cannot say that the candidate is a token because it has 2 separators in it
            return None
        }
        j+=1;
        if j == content.len() { // It means that we have seen every data from content and the possible token is never closed
            return None
        }
    }
    // Verify positions
    let cond1 = sep_position < j; // separator is always before the end of the token
    let cond2 = sep_position > 0 && sep_position < j-1; // we can't have "<;123...>" and "<123...;>"

    if !cond1 || !cond2 {
        return None
    }
    // Recover values and build token
    let offset : usize = String::from_utf8(content[..sep_position].to_vec()).unwrap().parse().unwrap();
    let length : usize = String::from_utf8(content[sep_position+1..j].to_vec()).unwrap().parse().unwrap();
    Some(DecompToken::new(offset, length))
}

fn extract(f_content : Vec<u8>, mut output : File, i : usize) -> Result<()>{ /// "i" parameter is the position after og file extension extraction
    Ok(())
}

pub fn decomp(path_to_file : &Path) -> Result<()>{
    Ok(())
}

impl DecompToken {
    fn new(o : usize, l : usize) -> Self{
        DecompToken {
            offset: o as usize,
            length: l as usize
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_gives_token(){
        // Basic
        let case1 = vec![49u8, 49, 54, 59, 49, 49, 56, 62];
        let t = gives_token(&case1);
        assert_eq!(t.is_some(), true);
        let token = t.unwrap();
        assert_eq!(token.offset, 116);
        assert_eq!(token.length, 118);
        // Not Tokens
        let case2 = vec![59, 49, 62];
        let t2 = gives_token(&case2);
        assert_eq!(t2.is_none(), true);
        let case3 = vec![49, 59, 62];
        let t3 = gives_token(&case3);
        assert_eq!(t3.is_none(), true);
        let case4 = vec![59, 59, 62];
        let t4 = gives_token(&case4);
        assert_eq!(t4.is_none(), true);
        let case5 = vec![49, 59, 49];
        let t5 = gives_token(&case5);
        assert_eq!(t5.is_none(), true);

    }
}
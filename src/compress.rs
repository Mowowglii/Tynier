use anyhow::Result;
use std::collections::VecDeque;
use std::fs::File;
use std::io::Write;

enum Decision {
    TakeToken(Token),
    KeepChunkf,
}

struct Token {
    data: Vec<u8>,
    offset: usize,
    replacement_length: usize,
    size: usize,
}

impl Token {
    pub fn new(offset_len_tuple: (usize, usize)) -> Self {
        // Encode the Token
        let mut d: VecDeque<u8> = VecDeque::new();
        // Encode delimiter (open)
        for del1 in "<".as_bytes() {
            d.push_back(*del1);
        }
        // Encode offset
        let add_one_offset = if offset_len_tuple.0 % 255 == 0 {
            0usize
        } else {
            1usize
        };
        for i in 0..(offset_len_tuple.0 / 255usize) + add_one_offset {
            if i == offset_len_tuple.0 / 255usize { // We enter into this statement if and only if add_one_offset = 1
                d.push_back((offset_len_tuple.0 % 255usize) as u8);
            } else { // Otherwise we just push 255
                d.push_back(255u8);
            }
        }
        // Encode separator
        for sep in ";".as_bytes() {
            d.push_back(*sep);
        }
        // Encode the length
        let add_one_length=  if offset_len_tuple.1%255usize == 0 {
            0usize
        } else {
            1usize
        };
        for j in 0..(offset_len_tuple.1 / 255usize) + add_one_length { // Works the same as encode offset
            if j == offset_len_tuple.1 / 255usize {
                d.push_back((offset_len_tuple.1 % 255usize) as u8);
            } else {
                d.push_back(255u8);
            }
        }
        // Encode delimiter (close)
        for del2 in ">".as_bytes() {
            d.push_back(*del2);
        }

        // Change data to get a Vec<u8>
        let final_d: Vec<u8> = d.into_iter().collect();

        // Save data length
        let s: usize = final_d.len();

        Token {
            data: final_d,
            offset: offset_len_tuple.0,
            replacement_length: offset_len_tuple.1,
            size: s,
        }
    }

    pub fn get_datas(&self) -> &[u8] {
        self.data.as_slice()
    }

    pub fn get_size(&self) -> usize {
        self.size
    }

    pub fn get_offset(&self) -> usize {
        self.offset
    }

    pub fn get_rep_length(&self) -> usize {
        self.replacement_length
    }
}

struct SlidingWindow {
    search_buffer: VecDeque<u8>,
    look_ahead_buffer: VecDeque<u8>,
    on: Vec<u8>,
    curr_byte: usize,
}

impl SlidingWindow {
    pub fn new(capacity: usize, buffer: Vec<u8>) -> Self {
        if capacity % 4 == 0 {
            SlidingWindow {
                search_buffer: VecDeque::with_capacity(capacity*3usize / 4usize),
                look_ahead_buffer: VecDeque::with_capacity(capacity / 4usize),
                on: buffer,
                curr_byte: 0usize,
            }
        } else {
            SlidingWindow {
                search_buffer: VecDeque::with_capacity(((capacity - (capacity % 4usize))*3usize / 4usize) + capacity % 4usize),
                look_ahead_buffer: VecDeque::with_capacity((capacity - (capacity % 4usize)) / 4usize),
                on: buffer,
                curr_byte: 0usize,
            }
        }
    }

    pub fn init(&mut self) -> Result<()> {
        // While there is data in self.on and look ahead buffer is not entirely full
        while self.curr_byte < self.on.len()
            && self.look_ahead_buffer.len() < self.look_ahead_buffer.capacity()
        {
            // Slide from data to look_ahead
            self.look_ahead_buffer
                .push_back(*(self.on.get(self.curr_byte).unwrap()));
            self.curr_byte += 1;
        }
        Ok(())
    }

    pub fn slide(&mut self) -> Option<u8> {
        match self.curr_byte + 1usize < self.on.len() {
            // Is there a next byte ?
            true => {
                match self.look_ahead_buffer.len() == self.look_ahead_buffer.capacity() {
                    // the Sliding Window must be initialized properly
                    true => {
                        match self.search_buffer.len() < self.search_buffer.capacity() {
                            // The search buffer is not full
                            true => {
                                // We could just return the value that quit the look ahead buffer
                                // We recover the value from look ahead buffer
                                let byte = &(self.look_ahead_buffer.pop_front().unwrap());

                                // We slide value from look_ahead to search
                                self.search_buffer.push_back(*byte);

                                // Slide from data to the look ahead
                                self.look_ahead_buffer
                                    .push_back(*(self.on.get(self.curr_byte + 1usize).unwrap()));

                                // Update the current byte
                                self.curr_byte += 1usize;

                                Some(*byte)
                            }
                            false => {
                                // We just have to slide everything
                                // Recover byte from look ahead buffer
                                let byte = &(self.look_ahead_buffer.pop_front().unwrap());

                                self.search_buffer.pop_front().unwrap(); // We can throw away the byte from the search buffer

                                // Slide from look ahead to search
                                self.search_buffer.push_back(*byte);

                                // Slide from data to the look ahead
                                self.look_ahead_buffer
                                    .push_back(*(self.on.get(self.curr_byte + 1usize).unwrap()));

                                // Update the current byte
                                self.curr_byte += 1usize;

                                Some(*byte)
                            }
                        }
                    }
                    false => {
                        panic!("Look Ahead Buffer not initialized properly !!!!");
                    }
                }
            }
            false => {
                if !(self.look_ahead_buffer.is_empty()) {
                    // We just have to slide the window without filling the look ahead buffer
                    let byte = &(self.look_ahead_buffer.pop_front().unwrap());

                    // Throw byte from search buffer
                    self.search_buffer.pop_front();

                    self.search_buffer.push_back(*byte); // slide from look ahead to search

                    Some(*byte)
                } else {
                    None
                }
            }
        }
    }

    pub fn jmp(&mut self, by: usize) -> Result<()> {
        for _ in 0..by {
            self.slide(); // We'll just throw away the values returned by slide because this function will be called for token injection
        }
        Ok(())
    }

    fn max(one: (usize, usize), other: (usize, usize)) -> (usize, usize) {
        if one.1 > other.1 {
            return one;
        } else {
            return other;
        }
    }

    fn get_offset_len(
        search_chunk: &VecDeque<u8>,
        la_chunk: &VecDeque<u8>,
        i: usize,
        j: usize,
        mut i_candidate: usize,
        mut total_match: usize,
    ) -> (usize, usize) {
        if i >= search_chunk.len() || j >= la_chunk.len() {
            // Convert i_candidate to the relative position
            let index = search_chunk.len() - i_candidate;
            return (index, total_match);
        }

        if search_chunk.get(i) == la_chunk.get(j) && total_match == 0 {
            i_candidate = i;
            total_match += 1;
            return SlidingWindow::get_offset_len(
                search_chunk,
                la_chunk,
                i + 1,
                j + 1,
                i_candidate,
                total_match,
            );
        }

        if search_chunk.get(i) == la_chunk.get(j) {
            return SlidingWindow::get_offset_len(
                search_chunk,
                la_chunk,
                i + 1,
                j + 1,
                i_candidate,
                total_match + 1,
            );
        } else {
            // Convert i_candidate to the relative position
            let index = search_chunk.len() - i_candidate;
            return SlidingWindow::max(
                (index, total_match),
                SlidingWindow::get_offset_len(search_chunk, la_chunk, i + 1, 0, i_candidate, 0),
            );
        }
    }

    pub fn build_token(&self) -> Token {
        // If one of the analysis buffers is empty, we just return a (0,0) token
        if self.search_buffer.len() == 0 {
            return Token::new((0usize, 0usize));
        }

        if self.look_ahead_buffer.len() == 0 {
            return Token::new((0usize, 0usize));
        }
        // In other case we calculate the token for the current state of both analysis buffer
        Token::new(SlidingWindow::get_offset_len(
            &self.search_buffer,
            &self.look_ahead_buffer,
            0usize,
            0usize,
            0usize,
            0usize,
        ))
    }

    pub fn decide(&self, token: Token) -> Decision {
        match token.get_size() < token.get_rep_length() {
            true => Decision::TakeToken(token),
            false => Decision::KeepChunkf,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.look_ahead_buffer.is_empty()
    }

    pub fn compress(&mut self, mut file: &File, extension : String) -> Result<()> {
        // f is a file that is opened in write only mode
        // We also need to encode the original file extension as header of the file
        file.write_all(extension.as_bytes());
        file.write("\n".as_bytes());
        // We init the SlidingWindow
        self.init()?;
        // We init the variable that will recover values from the file
        let mut byte: u8;
        while !(self.is_empty()) {
            // Doesn't stop while the look ahead buffer is not empty
            // We construct the token for the current state of both queues
            let token = self.build_token();
            // We decide...
            match self.decide(token) {
                Decision::TakeToken(t) => {
                    // We write the token found into the file
                    file.write_all(t.get_datas())?;
                    // We jump slide by the length of the data replaced
                    self.jmp(t.get_rep_length())?;
                }
                Decision::KeepChunkf => {
                    // We slide the window and we recover the byte from the file
                    byte = self.slide().unwrap();
                    // We can write that byte in the file
                    file.write(&[byte])?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fhandler::{generate_output, get_file_data};
    use std::{
        fs,
        io::{Read, Write},
        usize,
    };
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
    fn test_compress() {
        // Create a temporary file
        let binding = create_tmp_file(
            "compress_test",
            "I AM SAM. I AM SAM. SAM I AM.\nTHAT SAM-I-AM! THAT SAM-I-AM!\nI DO NOT LIKE THAT SAM-I-AM!\nDO WOULD YOU LIKE GREEN EGGS AND HAM?\nI DO NOT LIKE THEM,SAM-I-AM.\nI DO NOT LIKE GREEN EGGS AND HAM.",
        );
        let path = binding.path();
        let mut content: Vec<u8> = Vec::new();
        // Recover file datas
        let res1 = get_file_data(path, &mut content);
        // Make sure we recovered correctly the file data
        assert_eq!(res1.is_ok(), true);
        assert_eq!(content.len() > 0, true);

        let base_size = content.len() as u64; // Save the size of the file before compression

        // We create the SlidingWindow
        let mut sw = SlidingWindow::new(175, content);

        // We generate the output file
        let res2 = generate_output(path, false, None); // We generate the output
        assert_eq!(res2.is_ok(), true); // Make sure we could generate the output file

        // Compressing part
        let (file, ext) = res2.unwrap().unwrap(); // recover the file
        let final_res = sw.compress(&file, ext); // Compress the file
        assert_eq!(final_res.is_ok(), true); // Make sure the compression didn't failed
        // Recover size of the file after compression
        assert_eq!(base_size >= file.metadata().unwrap().len(), true);
    }

    #[test]
    fn get_ol_test() {
        let first = VecDeque::from([1u8, 2, 3, 4, 5, 6, 7, 8]);
        let seconds = VecDeque::from([4, 5, 6, 8, 7]);
        let result = SlidingWindow::get_offset_len(&first, &seconds, 0, 0, 0, 0);
        assert_eq!(result, (5, 3))
    }
}

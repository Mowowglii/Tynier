use anyhow::Result;
use std::collections::VecDeque;
use std::fs::File;
use std::io::Write;

enum Decision {
    TakeToken(Token),
    KeepChunkf,
}
#[derive(Debug)]
struct Token {
    data: Vec<u8>,
    replacement_length: usize,
    size: usize,
}

impl Token {
    pub fn new(offset_len_tuple: (usize, usize)) -> Self {
        // Encode the Token
        let mut d: Vec<u8> = Vec::new();
        // Encode delimiter (open)
        d.push(60u8); // push "<"
        // Encode offset if not 0
        if offset_len_tuple.0 == 0usize {
            d.push(0);
        } else {
            // Recover the significant bytes from the le bytes representation
            let offset_le_bytes : [u8; 8] = offset_len_tuple.0.to_le_bytes();
            for i in 1usize..9usize{
                if offset_le_bytes[8-i] != 0{
                    // Push the length of the offset le bytes representation
                    d.push(8u8-u8::try_from(i).ok().unwrap()+1); // Because i is strictly between 1 and 8, this conversion should work
                    // Push the offset le bytes representation
                    for byte in &offset_le_bytes[..9-i]{
                        d.push(*byte);
                    }
                    break;
                }
            }

        }
        // Encode separator
        d.push(59u8); // push ";"
        // Encode the length if not 0
        if offset_len_tuple.1 == 0 {
            d.push(0);
        } else {
            // Recover the significant bytes from the le bytes representation
            let length_le_bytes : [u8; 8] = offset_len_tuple.1.to_le_bytes();
            for i in 1usize..9usize{
                if length_le_bytes[8-i] != 0{
                    // Push the length of the offset le bytes representation
                    d.push(8u8-u8::try_from(i).ok().unwrap()+1); // Because i is strictly between 1 and 8, this conversion should work
                    // Push the offset le bytes representation
                    for byte in &length_le_bytes[..9-i]{
                        d.push(*byte);
                    }
                    break;
                }
            }
        }
        // Encode delimiter (close)
        d.push(62u8); // push ">"

        // Save data length
        let s: usize = d.len();

        Token {
            data: d,
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

    pub fn get_rep_length(&self) -> usize {
        self.replacement_length
    }
}

pub struct SlidingWindow {
    search_buffer: VecDeque<u8>,
    look_ahead_buffer: VecDeque<u8>,
    on: Vec<u8>,
    next_byte: usize,
}

impl SlidingWindow {
    pub fn new(capacity: usize, buffer: Vec<u8>) -> Self {
        if capacity.is_multiple_of(4) {
            SlidingWindow {
                search_buffer: VecDeque::with_capacity(capacity * 3usize / 4usize),
                look_ahead_buffer: VecDeque::with_capacity(capacity / 4usize),
                on: buffer,
                next_byte: 0usize,
            }
        } else {
            SlidingWindow {
                search_buffer: VecDeque::with_capacity(
                    ((capacity + (capacity % 4usize)) * 3usize / 4usize) + capacity % 4usize,
                ),
                look_ahead_buffer: VecDeque::with_capacity(
                    (capacity + (capacity % 4usize)) / 4usize,
                ),
                on: buffer,
                next_byte: 0usize,
            }
        }
    }

    pub fn init(&mut self) -> Result<()> {
        // While there is data in self.on and look ahead buffer is not entirely full
        while self.next_byte < self.on.len()
            && self.look_ahead_buffer.len() < self.look_ahead_buffer.capacity()
        {
            self.look_ahead_buffer.push_back(self.on[self.next_byte]);
            self.next_byte += 1;
        }
        Ok(())
    }

    pub fn slide(&mut self) -> Option<u8> {
        // Is there a next byte ?
        if self.next_byte < self.on.len() {
            // Is the search buffer already full ?
            if self.search_buffer.len() == self.search_buffer.capacity() {
                // Throw the latest byte from search buffer
                self.search_buffer.pop_front();
            }
            // Recover next byte on look ahead buffer
            let byte: &u8 = &(self.look_ahead_buffer.pop_front().unwrap());

            // Give it to the search buffer
            self.search_buffer.push_back(*byte);

            // Fill the look ahead buffer
            self.look_ahead_buffer.push_back(self.on[self.next_byte]);

            // Update the next byte index
            self.next_byte += 1;

            // Return the byte
            Some(*byte)
        } else {
            // Is the look ahead buffer empty ?
            if self.is_empty() {
                return None;
            }

            // Is the search buffer already full ?
            if self.search_buffer.len() == self.search_buffer.capacity() {
                // Throw the latest byte from search buffer
                self.search_buffer.pop_front();
            }

            // Recover next byte on look ahead buffer
            let byte: &u8 = &(self.look_ahead_buffer.pop_front().unwrap());

            // Give it to the search buffer
            self.search_buffer.push_back(*byte);

            // Update the next byte index
            self.next_byte += 1;

            // Return the byte
            Some(*byte)
        }
    }

    pub fn jmp(&mut self, by: usize) -> Result<()> {
        for _ in 0..by {
            self.slide(); // We'll just throw away the values returned by slide because this function will be called for token injection
        }
        Ok(())
    }

    fn max(one: (usize, usize), other: (usize, usize)) -> (usize, usize) {
        if one.1 > other.1 { one } else { other }
    }

    fn get_offset_len(search_chunk: &VecDeque<u8>, la_chunk: &VecDeque<u8>) -> (usize, usize) {
        let mut curr_best: (usize, usize) = (0, 0);

        let mut total_match = 0usize;
        let mut i_candidate = 0usize;
        let mut i = 0usize;
        let mut j = 0usize;

        while i < search_chunk.len() && j < la_chunk.len() {
            if search_chunk.get(i) == la_chunk.get(j) && total_match == 0 {
                // Set new candidate
                i_candidate = i;
                // Increment total_match of this one
                total_match += 1;
                // Looking next value in la_chunk
                j += 1;
            } else if search_chunk.get(i) == la_chunk.get(j) && total_match != 0 {
                // Increment the total match of previously candidate found
                total_match += 1;
                // Looking next value in la_chunk
                j += 1;
            } else if search_chunk.get(i) != la_chunk.get(j) {
                // Build candidate
                let candidate = (search_chunk.len() - i_candidate, total_match);
                // Save the best candidate
                curr_best = SlidingWindow::max(curr_best, candidate);
                // Reset
                j = 0;
                total_match = 0;
            }
            i += 1;
        }
        curr_best
    }

    fn build_token(&self) -> Token {
        // If one of the analysis buffers is empty, we just return a (0,0) token
        if self.search_buffer.is_empty() {
            return Token::new((0usize, 0usize));
        }

        if self.look_ahead_buffer.is_empty() {
            return Token::new((0usize, 0usize));
        }
        // In other case we calculate the token for the current state of both analysis buffer
        Token::new(SlidingWindow::get_offset_len(
            &self.search_buffer,
            &self.look_ahead_buffer,
        ))
    }

    fn decide(&self, token: Token) -> Decision {
        match token.get_size() < token.get_rep_length() {
            true => Decision::TakeToken(token),
            false => Decision::KeepChunkf,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.look_ahead_buffer.is_empty()
    }

    pub fn compress(&mut self, mut file: &File, extension: String) -> Result<()> {
        // f is a file that is opened in write only mode
        // Init the batch
        let mut batch: Vec<u8> = Vec::new();

        // Add the extension as the header
        for b in extension.as_bytes() {
            batch.push(*b);
        }

        batch.push(10u8);
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
                    // Push token to the batch
                    for byte in t.get_datas() {
                        batch.push(*byte);
                    }

                    // Jump the replaced text
                    self.jmp(t.get_rep_length())?;
                }
                Decision::KeepChunkf => {
                    // We slide the window and we recover the byte from the file
                    byte = self.slide().unwrap();

                    // We push that byte into the batch
                    batch.push(byte);
                }
            }

            // When batch contains more than 30 bytes, we can write it in the file and clear the batch
            if batch.len() >= 30 {
                file.write_all(&batch)?;
                batch.clear();
            };
        }
        // When we finish reading the file, we can write the last content of the batch
        if batch.len() > 0 {
            file.write_all(&batch)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fhandler::{generate_output, get_file_data};
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
        let mut sw = SlidingWindow::new(180, content);

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
        let result = SlidingWindow::get_offset_len(&first, &seconds);
        assert_eq!(result, (5, 3))
    }

    #[test]
    fn test_init() {
        let buff = vec![24u8, 15, 32, 34, 78, 89, 90, 245, 66, 79, 80, 32];
        let test = vec![24, 15, 32, 34, 78, 89, 90, 245, 66, 79, 80, 32];
        let mut sw = SlidingWindow::new(12usize, buff);
        let init_rest = sw.init();
        assert_eq!(init_rest.is_ok(), true); // Verifying the result of process
        assert_eq!(sw.look_ahead_buffer.len(), sw.look_ahead_buffer.capacity()); // Verifying the length of look ahead buffer
        for i in 0..sw.look_ahead_buffer.len() {
            assert_eq!(sw.look_ahead_buffer.pop_front(), Some(test[i])); // Verifying that correct values are sent to the look ahead buffer
        }
    }

    #[test]
    fn test_slide() {
        let buff = vec![24u8, 15, 32, 34, 78, 89, 90, 245, 66, 79, 80, 32];
        let test = vec![24, 15, 32, 34, 78, 89, 90, 245, 66, 79, 80, 32];
        let mut sw = SlidingWindow::new(12, buff);
        let init_rest = sw.init();
        assert_eq!(init_rest.is_ok(), true);
        // Normal Slide test
        for i in 0..9 {
            assert_eq!(sw.next_byte, 3 + i); // Verifying if Sliding Window index is sync with file index
            sw.slide();
            assert_eq!(sw.look_ahead_buffer.back(), Some(&test[3 + i])); // Verifying that we push correct value in look ahead buffer
        }
        let mut sw2 = SlidingWindow::new(12, test);
        let init_test = sw2.init();
        assert_eq!(init_test.is_ok(), true);
        // Jump test
        let jmp_test = sw2.jmp(5);
        assert_eq!(jmp_test.is_ok(), true);
        assert_eq!(sw2.next_byte, 8); // Checking that the current byte is updated correctly
    }
}

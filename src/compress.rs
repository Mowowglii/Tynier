use anyhow::Result;
use std::collections::VecDeque;

enum Decision{
    TakeToken(Token),
    KeepChunkf
}

struct Token {
    delim : i8,
    offset : usize,
    length : usize,
    sep : i8,
    size : usize
}

impl Token {
    pub fn new(offset_value : usize, length_value : usize) -> Self{
        Token {
            delim: -1i8,
            offset : offset_value,
            length : length_value,
            sep: -2i8,
            // Size of a token is the sizes of : the two delimiters + one separator + offset + length
            size: size_of_val(&offset_value) + size_of_val(&length_value) + 2*size_of_val(&-1i8) + size_of_val(&-2i8)
        }
    }

    pub fn get_size(&self) -> usize{
        self.size
    }

    pub fn get_offset(&self) -> usize{
        self.offset
    }

    pub fn get_length(&self) -> usize{
        self.length
    }
}

struct SlidingWindow {
    search_buffer : VecDeque<u8>,
    look_ahead_buffer : VecDeque<u8>,
    on : Box<Vec<u8>>,
    curr_byte : usize
}

impl SlidingWindow {
    pub fn new(max_capacity : usize, buffer : Box<Vec<u8>>) -> Self {
        if max_capacity % 2 == 1 {
            SlidingWindow {
                search_buffer: VecDeque::with_capacity(((max_capacity-1usize)/2usize)+1usize),
                look_ahead_buffer: VecDeque::with_capacity((max_capacity-1usize)/2usize),
                on : buffer,
                curr_byte : 0usize
            }
        } else {
            SlidingWindow {
                search_buffer: VecDeque::with_capacity(max_capacity/2usize),
                look_ahead_buffer: VecDeque::with_capacity(max_capacity/2usize),
                on : buffer,
                curr_byte : 0usize
            }
        }
    }

    pub fn init(&mut self) -> Result<()>{
         // While data is not entirely seen and look ahead buffer is not entirely full
        while self.curr_byte <= self.on.len() - 1usize && self.curr_byte <= self.look_ahead_buffer.capacity() - 1usize {
            // Slide from data to look_ahead
            self.look_ahead_buffer.push_back(*(self.on.get(self.curr_byte).unwrap()));
            // Update the curr_byte
            self.curr_byte += 1;
        }
        Ok(())
    }

    pub fn slide(&mut self) -> Option<u8> {
        match self.curr_byte + 1usize < self.on.len() { // Is there a next byte ?
            true => {
                match self.look_ahead_buffer.len() == self.look_ahead_buffer.capacity() { // the Sliding Window must be initialized properly
                    true => {
                        match self.search_buffer.len() < self.search_buffer.capacity() { // The search buffer is not full 
                            true => { // We have to fill the search buffer before
                                // We slide value from look_ahead to search
                                self.search_buffer.push_back(self.look_ahead_buffer.pop_front().unwrap());

                                // Slide from data to the look ahead
                                self.look_ahead_buffer.push_back(*(self.on.get(self.curr_byte+1usize).unwrap()));
                                // Update the current byte
                                self.curr_byte += 1usize;
                            
                                return None
                            },
                            false => { // We just have to slide everything
                                // Recover first byte
                                let f_byte = self.search_buffer.pop_front().unwrap();

                                // Slide from look ahead to search
                                self.search_buffer.push_back(self.look_ahead_buffer.pop_front().unwrap());

                                // Slide from data to the look ahead
                                self.look_ahead_buffer.push_back(*(self.on.get(self.curr_byte+1usize).unwrap()));
                                // Update the current byte
                                self.curr_byte += 1usize;
                            
                                return Some(f_byte)
                            }
                        }
                    },
                    false => {
                        panic!("Look Ahead Buffer not initialized properly !!!!");
                    }
                }
            },
            false => {
                match self.look_ahead_buffer.is_empty() {
                    true => {
                        if !(self.search_buffer.is_empty()){ // search buffer is not empty
                            return Some(self.search_buffer.pop_front().unwrap()) // So we emptying it...
                        } else {
                            return None // ... Already empty
                        }
                    },
                    false => { // In this case, because of the way we implemented the sliding window :
                        // It is impossible that the data buffer and the search buffer are empty but the look ahead buffer is not.
                        let f_byte = self.search_buffer.pop_front().unwrap();

                        // We slide value from look ahead to search
                        self.search_buffer.push_back(self.look_ahead_buffer.pop_front().unwrap());

                        // then we return the first byte that entered the sliding window
                        return Some(f_byte)
                    }
                }
            }
        }
    }

    pub fn jmp(&mut self, by : usize) -> Result<()>{
        for j in 0..by {
            self.slide(); // We'll just throw away the values returned by slide because this function will be called for token injection
        }
        Ok(())
    }

    pub fn build_token(&self) -> Token{
        panic!("Not implemented yet !");
    }

    pub fn decide(&self, token_size : usize, original_chunk_size : usize) -> Decision{
        panic!("Not implemented yet !");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
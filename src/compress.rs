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
         // While there is data in self.on and look ahead buffer is not entirely full
        while self.curr_byte+1usize < self.on.len() && self.look_ahead_buffer.len() < self.look_ahead_buffer.capacity() {
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
                            true => { // We could just return the value that quit the look ahead buffer
                                // We recover the value from look ahead buffer
                                let byte = &(self.look_ahead_buffer.pop_front().unwrap());

                                // We slide value from look_ahead to search
                                self.search_buffer.push_back(*byte);

                                // Slide from data to the look ahead
                                self.look_ahead_buffer.push_back(*(self.on.get(self.curr_byte+1usize).unwrap()));
                                
                                // Update the current byte
                                self.curr_byte += 1usize;
                            
                                Some(*byte)
                            },
                            false => { // We just have to slide everything
                                // Recover byte from look ahead buffer
                                let byte = &(self.look_ahead_buffer.pop_front().unwrap());

                                self.search_buffer.pop_front().unwrap(); // We can throw away the byte from the search buffer

                                // Slide from look ahead to search
                                self.search_buffer.push_back(*byte);

                                // Slide from data to the look ahead
                                self.look_ahead_buffer.push_back(*(self.on.get(self.curr_byte+1usize).unwrap()));
                                
                                // Update the current byte
                                self.curr_byte += 1usize;
                            
                                Some(*byte)
                            }
                        }
                    },
                    false => {
                        panic!("Look Ahead Buffer not initialized properly !!!!");
                    }
                }
            },
            false => {
                if !(self.look_ahead_buffer.is_empty()){    
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

    pub fn jmp(&mut self, by : usize) -> Result<()>{
        for _ in 0..by {
            self.slide(); // We'll just throw away the values returned by slide because this function will be called for token injection
        }
        Ok(())
    }

    fn get_offset_len(search_chunk : &VecDeque<u8>, la_chunk : &VecDeque<u8>) -> (usize,usize){
        panic!("Not implemented yet !")
    }

    pub fn build_token(&self) -> Token{
        panic!("Not implemented yet !");
        if self.search_buffer.len() == 0 {
            return Token::new(0usize, 0usize)
        }

        if self.look_ahead_buffer.len() == 0{
            return Token::new(0usize, 0usize)
        }
        Token::new(0usize, 0usize)
    }

    pub fn decide(&self, token : Token) -> Decision{
        panic!("Not implemented yet !");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
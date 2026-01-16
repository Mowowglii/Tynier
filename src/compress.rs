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
    pub fn new(offset_len_tuple : (usize, usize)) -> Self{
        Token {
            delim: -1i8,
            offset : offset_len_tuple.0,
            length : offset_len_tuple.1,
            sep: -2i8,
            // Size of a token is the sizes of : the two delimiters + one separator + offset + length
            size: size_of_val(&offset_len_tuple) + size_of_val(&offset_len_tuple) + 2*size_of_val(&-1i8) + size_of_val(&-2i8)
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

    fn max(one : (usize, usize), other : (usize, usize)) -> (usize, usize){
        if one.1 > other.1 {
            return one
        } else {
            return other
        }
    }

    fn get_offset_len(search_chunk : &VecDeque<u8>, la_chunk : &VecDeque<u8>, i : usize, j : usize, mut i_candidate : usize, mut total_match : usize) -> (usize,usize){
        if i >= search_chunk.len() || j >= la_chunk.len(){
            // Convert i_candidate to the relative position
            let index = search_chunk.len() - i_candidate;
            return (index , total_match)
        }

        if search_chunk.get(i) == la_chunk.get(j) && total_match == 0{
            i_candidate = i;
            total_match += 1;
            return SlidingWindow::get_offset_len(search_chunk, la_chunk, i+1, j+1, i_candidate, total_match)
        }

        if search_chunk.get(i) == la_chunk.get(j){
            return SlidingWindow::get_offset_len(search_chunk, la_chunk, i+1, j+1, i_candidate, total_match+1)
        } else {
            // Convert i_candidate to the relative position
            let index = search_chunk.len() - i_candidate;
            return SlidingWindow::max((index, total_match), SlidingWindow::get_offset_len(search_chunk, la_chunk, i+1, 0, i_candidate, 0))
        }
    }

    pub fn build_token(&self) -> Token{
        // If one of the analysis buffers is empty, we just return a (0,0) token
        if self.search_buffer.len() == 0 { 
            return Token::new((0usize, 0usize))
        }

        if self.look_ahead_buffer.len() == 0{
            return Token::new((0usize, 0usize))
        }
        // In other case we calculate the token for the current state of both analysis buffer
        Token::new(SlidingWindow::get_offset_len(&self.search_buffer, &self.look_ahead_buffer, 0usize, 0usize, 0usize, 0usize))
    }

    pub fn decide(&self, token : Token) -> Decision{
        panic!("Not implemented yet !");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_ol_test(){
        let first = VecDeque::from([1u8, 2, 3, 4, 5, 6, 7, 8]);
        let seconds = VecDeque::from([4, 5, 6, 8, 7]);
        let result = SlidingWindow::get_offset_len(&first, &seconds, 0, 0, 0, 0);
        assert_eq!(result, (5, 3))
    }
}
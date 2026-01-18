use anyhow::Result;
use std::collections::VecDeque;

enum Decision{
    TakeToken(Token),
    KeepChunkf
}

struct Token{
    data : VecDeque<u8>,
    offset : usize,
    replacement_length : usize,
    size : usize
}

impl Token{
    pub fn new(offset_len_tuple : (usize, usize)) -> Self{
    // Encode the Token
        let mut d : VecDeque<u8> = VecDeque::new();
        // Encode delimiter (open)
        for del1 in "::".as_bytes(){
            d.push_back(*del1);
        }
        // Encode offset
        for i in 0..(offset_len_tuple.0/255usize)+1usize{
            if (i == offset_len_tuple.0/255usize){
                d.push_back((offset_len_tuple.0%255usize) as u8);
            } else {
                d.push_back(255u8);
            }
        }
        // Encode separator
        for sep in ";;".as_bytes(){
            d.push_back(*sep);
        }
        // Encode the length
        for j in 0..(offset_len_tuple.1/255usize)+1usize{
            if (j == offset_len_tuple.1/255usize){
                d.push_back((offset_len_tuple.1%255usize) as u8);
            } else {
                d.push_back(255u8);
            }
        }
        // Encode delimiter (close)
        for del2 in "::".as_bytes(){
            d.push_back(*del2);
        }
        // Save data length
        let s : usize = d.len();

        Token {
            data: d,
            offset: offset_len_tuple.0,
            replacement_length: offset_len_tuple.1,
            size: s
        }
    }

    pub fn get_datas(&self) -> &VecDeque<u8>{
        &self.data
    }

    pub fn get_size(&self) -> usize{
        self.size
    }

    pub fn get_offset(&self) -> usize{
        self.offset
    }

    pub fn get_rep_length(&self) -> usize{
        self.replacement_length
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
        match token.get_size() < token.get_rep_length() {
            true => Decision::TakeToken(token),
            false => Decision::KeepChunkf
        }
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
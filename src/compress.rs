use anyhow::Result;
use std::collections::VecDeque;
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
}

struct SlidingWindow {
    search_buffer : VecDeque<u8>,
    look_ahead_buffer : VecDeque<u8>
}

impl SlidingWindow {
    pub fn new(max_capacity : usize) -> Self {
        let capa = if max_capacity % 2 == 1 {
            (max_capacity-1usize)/2usize
        } else {
            max_capacity/2usize
        };
        SlidingWindow {
            search_buffer: VecDeque::with_capacity(capa+1usize),
            look_ahead_buffer: VecDeque::with_capacity(capa)
        }
    }

    pub fn slide(&mut self, next_byte : &u8) -> Result<()> {
        // We are checking if the look_ahead_buffer is empty
        if self.look_ahead_buffer.is_empty() {
            self.look_ahead_buffer.push_back(*next_byte);
            return Ok(())
        }

        // We are checking if the search buffer is full
        if self.search_buffer.len() == self.search_buffer.capacity() {
            self.search_buffer.pop_front(); // We'll just throw it away
        }

        // Slide the first byte from look_ahead_buffer to the search_buffer
        self.search_buffer.push_back(self.look_ahead_buffer.pop_front().unwrap());

        // Add the next byte to look_ahead_buffer
        self.look_ahead_buffer.push_back(*next_byte);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
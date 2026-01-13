use anyhow::Result;
use std::collections::VecDeque;


struct SlidingWindow {
    search_buffer : VecDeque<u8>,
    look_ahead_buffer : VecDeque<u8>,
    max_sliding_window_size : usize,
}

impl SlidingWindow {
    pub fn new(with_capacity : usize) -> Self {
        SlidingWindow { 
            search_buffer: VecDeque::with_capacity(with_capacity/2usize),
            look_ahead_buffer: VecDeque::with_capacity(with_capacity/2usize),
            max_sliding_window_size: with_capacity 
        }
    }

    pub fn slide(&mut self, next_byte : u8) -> Result<()> {
        // We are checking if the look_ahead_buffer is empty
        if self.look_ahead_buffer.is_empty() {
            self.look_ahead_buffer.push_back(next_byte);
            return Ok(())
        }

        // We are checking if the search buffer is full
        if self.search_buffer.len() == self.search_buffer.capacity() {
            self.search_buffer.pop_front(); // We'll just throw it away
        }

        // Slide the first byte from look_ahead_buffer to the search_buffer
        self.search_buffer.push_back(self.look_ahead_buffer.pop_front().unwrap());

        // Add the next byte to look_ahead_buffer
        self.look_ahead_buffer.push_back(next_byte);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
}
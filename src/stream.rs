///
/// Stream functions
///


use std::clone::Clone;
use std::io::Write;
use std::io::Read;
use std::io;


struct Buffer {
    /// Buffer storage
    buff:         [u8; 1024],
    /// Number of bytes writen to the buffer
    used:         usize,
    /// Bytes upto this index are not part of the boundary
    non_boundary: usize,
    /// Number of matched bytes
    matched:      usize,
}


impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            buff: [0; 1024],
            used: 0,
            non_boundary: 0,
            matched: 0,
        }
    }
}


impl Clone for Buffer {
    fn clone(&self) -> Buffer {
        let mut buffer = Buffer::new();
        buffer.buff.clone_from_slice(&(self.buff));
        buffer.used = self.used;
        buffer.non_boundary = self.non_boundary;
        buffer.matched = self.matched;
        buffer
    }
}


struct NonWriter {
    // can't have an empty struct
    i_exist: bool,
}


impl Write for NonWriter {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        Ok(b.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}


/// Advances the stream until the boundary sequence has been reached, or
/// until max_len bytes have been read.
pub fn advance_stream(input:    &mut Read,
                      max_len:  usize,
                      boundary: String) -> io::Result<usize> {
    // Dirty hack
    let mut non_writer = NonWriter {i_exist: true};
    try!(write_stream(input, &mut non_writer, max_len, boundary));
    Ok(1)
}


/// Reads and advances the input stream until one of the conditions is met:
///   1. the boundary sequence is reached
///   2. max_len number of bytes have been read, or
///   3. the end of the stream has been reached.
///
/// Bytes that are read are written to output stream. The boundary
/// sequence, if found, is not written.
pub fn write_stream(input:    &mut Read,
                    output:   &mut Write,
                    max_len:  usize,
                    boundary: String) -> io::Result<usize>
{
    let boundary_buffer = boundary.as_bytes();
    let boundary_length = boundary.len();

    let mut write_buff: Buffer = Buffer::new();
    let mut queue_buff: Buffer = Buffer::new();

    let mut boundary_cursor: usize = 0;
    let mut read_total:      usize = 0;
    let mut write_total:     usize = 0;

    while read_total <= max_len {
        write_buff.used = input.read(&mut write_buff.buff).unwrap();
        write_buff.non_boundary = write_buff.used;
        read_total += write_buff.used;

        if write_buff.used == 0 {
            if queue_buff.used > 0 {
                write_total += try!(output.write(&queue_buff.buff[0..(queue_buff.used)]));
            }
            break;
        }

        for i in 0..(write_buff.used) {
            if boundary_cursor == boundary_length - 1 {
                break;
            }
            if write_buff.buff[i] == boundary_buffer[boundary_cursor] {
                boundary_cursor += 1;
                write_buff.matched += 1;
            } else {
                boundary_cursor = 0;
                write_buff.non_boundary = i;
                write_buff.matched = 0;
            }
        }

        if boundary_cursor == boundary_length - 1 {
            // We have a complete boundary match

            if write_buff.matched == boundary_length {
                // Case 1: the boundary match is in the write buffer
                //
                // Write buffer:
                // +---+---+---+---+---+---+
                // | 1 | 0 | B | A | R |   |
                // +---+---+---+---+---+---+
                //

                // Write previously queued buffer if one exists
                if queue_buff.used > 0 {
                    write_total += try!(output.write(&queue_buff.buff[0..(queue_buff.used)]));
                }
                write_total += try!(output.write(&write_buff.buff[0..(write_buff.non_boundary)]));
            }  else {
                // Case 2: the boundary match is split across the queued buffer and the write buffer.
                //
                // Queue buffer:                 Write buffer:
                // +---+---+---+---+---+---+     +---+---+---+---+---+---+
                // | 1 | 0 | 7 | k | o | B |     | A | R |   |   |   |   |
                // +---+---+---+---+---+---+     +---+---+---+---+---+---+
                //
                write_total += try!(output.write(&queue_buff.buff[0..(queue_buff.non_boundary)]));
            }
            break; // exit loop
        } else if boundary_cursor > 0 {
            // We have a partial boundary match in the write buffer
            //
            // Write buffer:
            // +---+---+---+---+---+---+
            // | 8 | n | q | f | B | A |
            // +---+---+---+---+---+---+
            //
            if queue_buff.used > 0 {
                write_total += try!(output.write(&queue_buff.buff[0..(queue_buff.used)]));
            }
            // copy write buffer to queue buffer
            queue_buff = write_buff.clone();
        } else {
            // No match found.
            if queue_buff.used > 0 {
                write_total += try!(output.write(&queue_buff.buff[0..(queue_buff.used)]));
            }
            write_total += try!(output.write(&write_buff.buff[0..(write_buff.used)]));
        }
    }
    Ok(write_total)
}

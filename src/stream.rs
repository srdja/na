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


pub struct WriteBuffer<'a> {
    buffer: &'a mut [u8],
    writen: usize,
}


impl<'a> WriteBuffer<'a> {
    pub fn new(b: &'a mut [u8]) -> WriteBuffer<'a> {
        WriteBuffer {
            buffer: b,
            writen: 0
        }
    }
}


impl<'a> Write for WriteBuffer<'a> {
    fn write(&mut self, buff: &[u8]) -> io::Result<usize> {
        let w = self.writen;
        let l = self.buffer.len();

        let mut writen = 0;
        for (to, from) in self.buffer[w..l].iter_mut().zip(buff.iter()) {
            writen += 1;
            *to = *from
        }
        self.writen += writen;
        Ok(writen)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}


pub struct ReadBuffer<'a> {
    buffer: &'a[u8],
    read:   usize,
}


impl<'a> ReadBuffer<'a> {
    pub fn new(b: &'a [u8]) -> ReadBuffer<'a> {
        ReadBuffer {
            buffer: b,
            read:   0,
        }
    }
}


impl<'a> Read for ReadBuffer<'a> {
    fn read(&mut self, buff: &mut [u8]) -> io::Result<usize> {
        let max_len = buff.len();
        let available_bytes = self.buffer.len() - self.read;

        let mut read = 0;
        let b = self.read;
        let e;
        if available_bytes > buff.len() {
            e = self.read + buff.len();
        } else {
            e = self.read + available_bytes;
        }
        for (from, to) in self.buffer[b..e].iter().zip(buff.iter_mut()) {
            read += 1;
            *to = *from
        }
        self.read += read;
        Ok(read)
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


#[test]
pub fn test_read_buffer() {
    let mut buff: [u8; 1000] = [42; 1000];
    buff[7  ] = 6;
    buff[299] = 3;
    buff[999] = 2;
    buff[300] = 5;

    let mut reader = ReadBuffer::new(&buff);
    let mut into: [u8; 300] = [0; 300];

    assert_eq!(300, reader.read(&mut into).unwrap());
    assert_eq!(3, into[299]);
    assert_eq!(6, into[7]);
    assert_eq!(300, reader.read(&mut into).unwrap());
    assert_eq!(5, into[0]);
    assert_eq!(300, reader.read(&mut into).unwrap());
    assert_eq!(100, reader.read(&mut into).unwrap());
    assert_eq!(2, into[99]);
}


#[test]
pub fn test_write_buffer() {
    let mut buffer: [u8; 1000] = [0; 1000];

    {
        let mut writer = WriteBuffer::new(&mut buffer);
        let mut input: [u8; 1000] = [42; 1000];
        input[0  ] = 1;
        input[299] = 2;
        input[300] = 3;
        input[999] = 4;

        assert_eq!(150, writer.write(&input[0..150]).unwrap());
        assert_eq!(177, writer.write(&input[150..327]).unwrap());
        assert_eq!(273, writer.write(&input[327..600]).unwrap());
        assert_eq!(300, writer.write(&input[600..900]).unwrap());
        assert_eq!(100, writer.write(&input[900..1000]).unwrap());
    }

    assert_eq!(1, buffer[0]);
    assert_eq!(2, buffer[299]);
    assert_eq!(3, buffer[300]);
    assert_eq!(4, buffer[999]);
}


#[test]
pub fn test_write_stream_case1() {

}

#[test]
pub fn test_write_stream_case2() {

}

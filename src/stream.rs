///
/// Stream parsing functions
///


use std::clone::Clone;
use std::io::Write;
use std::io::Read;
use std::io;


struct Buffer {
    buff:         [u8; 1024],
    /// Number of bytes written to the buffer
    used:         usize,
    /// Number of bytes that are not a part of the boundary
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



/// Reads and advances the input stream until one of the conditions is met:
///   1. the boundary sequence is reached
///   2. max_len number of bytes have been read, or
///   3. the end of the stream has been reached.
///
/// Bytes that are read are written to output stream. The boundary
/// sequence, if found, is not written.
pub fn write_stream(input:     &mut Read,
                    output:    &mut Write,
                    max_len:   usize,
                    boundary:  String) -> io::Result<usize>
{

    let boundary_buffer = boundary.as_bytes();
    let boundary_length = boundary.len();

    let mut write_buff: Buffer = Buffer::new();
    let mut queue_buff: Buffer = Buffer::new();

    let mut boundary_cursor: usize = 0;
    let mut read_total:      usize = 0;
    let mut write_total:     usize = 0;

    loop {
        write_buff.used = try!(input.read(&mut write_buff.buff));
        write_buff.non_boundary = 0;
        read_total += write_buff.used;

        // No more data
        if write_buff.used == 0 {
            if queue_buff.used > 0 {
                write_total += try!(output.write(&queue_buff.buff[0..(queue_buff.used)]));
            }
            break;
        }

        // Seek the boundary sequence
        for i in 0..(write_buff.used) {
            if boundary_cursor == boundary_length {
                break;
            }
            if write_buff.buff[i] == boundary_buffer[boundary_cursor] {
                boundary_cursor += 1;
                write_buff.matched += 1;
            } else {
                boundary_cursor = 0;
                // number of non boundary elements = index + 1
                write_buff.non_boundary = i + 1;
                write_buff.matched = 0;
            }
        }

        // Max len is reached before the boundary
       if (read_total >= max_len) && (max_len <= (write_total + write_buff.non_boundary)) {
            if queue_buff.used > 0 {
                write_total += try!(output.write(&queue_buff.buff[0..(queue_buff.used)]));
            }
            let upto = write_buff.used - (read_total - max_len);
            write_total += try!(output.write(&write_buff.buff[0..upto]));
            break;
        }

        if boundary_cursor == boundary_length {
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
                    queue_buff.used = 0;
                    queue_buff.matched = 0;
                    queue_buff.non_boundary = 0;
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
                queue_buff.used = 0;
                queue_buff.matched = 0;
                queue_buff.non_boundary = 0;
            }
            // copy write buffer to queue buffer
            queue_buff = write_buff.clone();
        } else {
            // No match found.
            if queue_buff.used > 0 {
                write_total += try!(output.write(&queue_buff.buff[0..(queue_buff.used)]));
                queue_buff.used = 0;
                queue_buff.matched = 0;
                queue_buff.non_boundary = 0;
            }
            write_total += try!(output.write(&write_buff.buff[0..(write_buff.used)]));
        }
    }
    Ok(write_total)
}


// Tests


#[allow(dead_code)]
pub struct WriteBuffer<'a> {
    buffer: &'a mut [u8],
    written: usize,
}


#[allow(dead_code)]
impl<'a> WriteBuffer<'a> {
    pub fn new(b: &'a mut [u8]) -> WriteBuffer<'a> {
        WriteBuffer {
            buffer: b,
            written: 0
        }
    }
}


#[allow(dead_code)]
impl<'a> Write for WriteBuffer<'a> {
    fn write(&mut self, buff: &[u8]) -> io::Result<usize> {
        let w = self.written;
        let l = self.buffer.len();

        let mut written = 0;
        for (to, from) in self.buffer[w..l].iter_mut().zip(buff.iter()) {
            written += 1;
            *to = *from
        }
        self.written += written;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}


#[allow(dead_code)]
pub struct ReadBuffer<'a> {
    buffer: &'a[u8],
    read:   usize,
}


#[allow(dead_code)]
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
    // Boundary at the beginning of the second read
    let boundary = "foobar".as_bytes();
    let decoy    = "fooba".as_bytes();

    let mut input:  [u8; 2048] = [0; 2048];
    let mut output: [u8; 2048] = [0; 2048];

    input[500] = decoy[0];
    input[501] = decoy[1];
    input[502] = decoy[2];
    input[503] = decoy[3];
    input[504] = decoy[4];

    input[1023] = 42;
    input[1024] = boundary[0];
    input[1025] = boundary[1];
    input[1026] = boundary[2];
    input[1027] = boundary[3];
    input[1028] = boundary[4];
    input[1029] = boundary[5];

    let w;
    {
        let mut reader = ReadBuffer::new(&input);
        let mut writer = WriteBuffer::new(&mut output);
        w = write_stream(&mut reader, &mut writer, 3000, "foobar".to_string()).unwrap();
    }
    assert_eq!(1024, w);
    assert_eq!(output[1023], 42);
    assert_eq!(output[1024], 0);
}


#[test]
pub fn test_write_stream_case2() {
    // Boundary at the ~middle of the second read
    let boundary = "foobar".as_bytes();
    let decoy    = "fooba".as_bytes();

    let mut input:  [u8; 2048] = [0; 2048];
    let mut output: [u8; 2048] = [0; 2048];

    input[1022] = decoy[0];
    input[1023] = decoy[1];
    input[1024] = decoy[2];
    input[1025] = decoy[3];
    input[1026] = decoy[4];

    input[1523] = 42;
    input[1524] = boundary[0];
    input[1525] = boundary[1];
    input[1526] = boundary[2];
    input[1527] = boundary[3];
    input[1528] = boundary[4];
    input[1529] = boundary[5];

    let w;
    {
        let mut reader = ReadBuffer::new(&input);
        let mut writer = WriteBuffer::new(&mut output);
        w = write_stream(&mut reader, &mut writer, 3000, "foobar".to_string()).unwrap();
    }
    assert_eq!(output[1523], 42);
    assert_eq!(1524, w);
    assert_eq!(output[1524], 0);
}


#[test]
pub fn test_write_stream_case3() {
    // boundary split across two reads
    let boundary = "foobar".as_bytes();

    let mut input:  [u8; 2048] = [0; 2048];
    let mut output: [u8; 2048] = [0; 2048];

    input[1520] = 42;
    input[1521] = boundary[0];
    input[1522] = boundary[1];
    input[1523] = boundary[2];
    input[1524] = boundary[3];
    input[1525] = boundary[4];
    input[1526] = boundary[5];

    let w;
    {
        let mut reader = ReadBuffer::new(&input);
        let mut writer = WriteBuffer::new(&mut output);
        w = write_stream(&mut reader, &mut writer, 3000, "foobar".to_string()).unwrap();
    }
    assert_eq!(1521, w);
    assert_eq!(output[1520], 42);
    assert_eq!(output[1521], 0);
}


#[test]
pub fn test_write_stream_case4() {
    // no boundary found
    let mut input:  [u8; 2048] = [0; 2048];
    let mut output: [u8; 2048] = [0; 2048];

    input[2047] = 42;

    let w;
    {
        let mut reader = ReadBuffer::new(&input);
        let mut writer = WriteBuffer::new(&mut output);
        w = write_stream(&mut reader, &mut writer, 3000, "foobar".to_string()).unwrap();
    }
    assert_eq!(2048, w);
    assert_eq!(output[2047], 42);
}


#[test]
pub fn test_write_stream_case5() {
    // max_len early exit
    let boundary = "foobar".as_bytes();

    let mut input:  [u8; 2048] = [0; 2048];
    let mut output: [u8; 2048] = [0; 2048];

    input[1449] = 42;
    input[1521] = boundary[0];
    input[1522] = boundary[1];
    input[1523] = boundary[2];
    input[1524] = boundary[3];
    input[1525] = boundary[4];
    input[1526] = boundary[5];

    let w;
    {
        let mut reader = ReadBuffer::new(&input);
        let mut writer = WriteBuffer::new(&mut output);
        w = write_stream(&mut reader, &mut writer, 1450, "foobar".to_string()).unwrap();
    }
    assert_eq!(1450, w);
    assert_eq!(output[1449], 42);
}


#[test]
pub fn test_write_stream_case6() {
    let mut input:  [u8; 2048] = [0; 2048];
    let mut output: [u8; 2048] = [0; 2048];

    input[199] = 42;
    input[200] = 84;

    let w;
    {
        let mut reader = ReadBuffer::new(&input);
        let mut writer = WriteBuffer::new(&mut output);
        w = write_stream(&mut reader, &mut writer, 200, "foobar".to_string()).unwrap();
    }
    assert_eq!(200, w);
    assert_eq!(output[199], 42);
    assert_eq!(output[200], 0)
}

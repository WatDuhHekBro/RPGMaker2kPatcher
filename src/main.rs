mod structs; // Main structures
mod types; // Custom data types
mod wrappers; // Intermediary data types for specialized functionality (upfront byte counts & null ID lists (similar to NullStrings))

use crate::structs::LcfMapUnit;
use binrw::{
    io::{Cursor, Seek, Write},
    BinRead, BinWrite, BinWriterExt,
};

fn main() {
    let mut reader = Cursor::new(include_bytes!(
        "/home/watduhhekbro/external/workspace/Map0134.lmu"
    ));
    let servers = LcfMapUnit::read(&mut reader).unwrap();
    println!("{servers:?}\n");

    let mut writer = Cursor::new(Vec::<u8>::new());
    writer.write_be(&servers).unwrap();
    println!("{:02X?}", writer.into_inner());

    // stream & map_stream test
    let mut out = Cursor::new(vec![]);
    //Test { a: 0x201, b: 0x403 }.write(&mut out).unwrap();
    // The map_stream/stream combo iterates over every byte of Test, but not TestParent.
    TestParent {
        version: 42069,
        blank: 69,
        yeet: Test {
            values: TestValues { a: 0x201, b: 0x403 },
        },
    }
    .write(&mut out)
    .unwrap();
    println!("\n{:?}", out.into_inner());
}

// The only example use of map_stream and stream is from a unit test
// https://github.com/jam1garner/binrw/blob/master/binrw/tests/derive/write/stream.rs

struct Checksum<T> {
    inner: T,
    check: core::num::Wrapping<u8>,
}

impl<T> Checksum<T> {
    fn new(inner: T) -> Self {
        Self {
            inner,
            check: core::num::Wrapping(0),
        }
    }

    fn check(&self) -> u8 {
        self.check.0
    }
}

impl<T: Write> Write for Checksum<T> {
    fn write(&mut self, buf: &[u8]) -> binrw::io::Result<usize> {
        println!("\nBuffer::write() = {buf:?}");
        for b in buf {
            print!("0x{b:X} ");
            //self.check += 1;
            self.check += 10;
            //self.check += b;
            print!("0x{b:X} ");
        }
        let result = self.inner.write(buf);
        println!("{result:?}");
        result

        //Ok(0x7F)
        //self.inner.write(&[0x0F, 0x7E])
    }

    fn flush(&mut self) -> binrw::io::Result<()> {
        self.inner.flush()
    }
}

impl<T: Seek> Seek for Checksum<T> {
    fn seek(&mut self, pos: binrw::io::SeekFrom) -> binrw::io::Result<u64> {
        self.inner.seek(pos)
    }
}

#[binrw::binwrite]
#[bw(little, stream = writer, map_stream = Checksum::new, import(seek: i64))]
struct Test {
    #[bw(restore_position)]
    values: TestValues,
    // Maybe pass the byte length check as a parameter to its parent?
    #[bw(calc(writer.check()), seek_before = binrw::io::SeekFrom::Current(seek))]
    c: u8,
}

#[binrw::binwrite]
#[bw(little)]
struct TestValues {
    a: u16,
    b: u16,
}

#[binrw::binwrite]
#[bw(little)]
struct TestParent {
    version: u32,
    // The parent will hold the byte length, initially just zeroes. If you add the "blank" parameter in the child, it'll be counted in the bytes.
    // Using a custom number parser variant (dynamic number + # of bytes for that number), two numbers will be read, the 2nd will be sent as an argument.
    blank: u8,
    // The child will then overwrite some of the parent's bytes in a controlled manner.
    #[bw(args(-1))]
    yeet: Test,
}

use crate::{structs::map::LcfMapUnitEvent, types::DynamicInteger};
use binrw::{
    binrw,
    io::{Cursor, Read, Seek, Write},
    BinRead, BinResult, BinWrite, BinWriterExt, Endian,
};
//use std::fmt;

#[binrw]
#[derive(Debug)]
struct Test {
    count: DynamicInteger,
}

#[derive(Debug)]
pub struct EventsWrapper(Test);

impl BinRead for EventsWrapper {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut bytes = vec![];
        let count = <DynamicInteger>::read_options(reader, endian, ())?;

        for _ in 0..*count {
            let byte = <u8>::read_options(reader, endian, ())?;
            bytes.push(byte);
        }

        println!(
            "EventsWrapper = {bytes:02X?}\nByte Count: {} = {}\n\n",
            bytes.len(),
            count
        );

        Ok(EventsWrapper(Test {
            count: DynamicInteger(bytes[0] as i32),
        }))
    }
}

impl BinWrite for EventsWrapper {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        /*let count: i32 = self.0.len().try_into().expect(&format!(
            "The length of the u8 array {:?} could not fit into an i32!",
            self.0
        ));*/
        //self.0.write_options(writer, endian, args)?;

        let mut w = Cursor::new(Vec::<u8>::new());
        w.write_be(&self.0).unwrap();
        let data = w.into_inner();
        //println!("{:02X?}", data);
        DynamicInteger(data.len().try_into().unwrap()).write_options(writer, endian, args)?;
        data.write_options(writer, endian, args)?;

        Ok(())
    }
}

/*impl fmt::Debug for EventsWrapper {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self.0)
    }
}

impl std::ops::Deref for EventsWrapper {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for EventsWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}*/

/*#[cfg(test)]
mod tests {
    use super::*;
    use binrw::{binrw, io::Cursor, BinWriterExt};

    #[binrw]
    #[br(big)]
    struct TestStructure(EventsWrapper);

    #[test]
    fn read_success() {
        let mut reader = Cursor::new(b"\x05Hello");
        let data = TestStructure::read(&mut reader).unwrap();
        assert_eq!(*data.0, b"Hello".to_vec());
    }

    #[test]
    fn write_success() {
        let data = TestStructure(EventsWrapper(b"Yeetus".to_vec()));
        let mut writer = Cursor::new(Vec::<u8>::new());
        writer.write_le(&data).unwrap();
        assert_eq!(writer.into_inner(), b"\x06Yeetus");
    }
}*/

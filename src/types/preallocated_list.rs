use crate::{types::DynamicInteger, util::constants::ERROR_BINRW_READ};
use binrw::{
    io::{Read, Seek, Write},
    BinRead, BinResult, BinWrite, Endian,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct PreallocatedList<T: BinRead>(pub Vec<T>);

impl<T: for<'a> BinRead<Args<'a> = ()>> BinRead for PreallocatedList<T> {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let count = DynamicInteger::read_be(reader).expect(ERROR_BINRW_READ);
        let mut entries: Vec<T> = Vec::with_capacity(count.0 as usize);

        for _ in 0..*count {
            let entry = T::read_options(reader, endian, args).expect(ERROR_BINRW_READ);
            entries.push(entry);
        }

        Ok(PreallocatedList(entries))
    }
}

impl<T: BinRead + BinWrite + 'static> BinWrite for PreallocatedList<T>
where
    for<'a> T: BinRead + BinWrite<Args<'a> = ()>,
{
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        // Write back the count before writing the rest
        DynamicInteger(self.0.len().try_into().unwrap()).write_options(writer, endian, args)?;
        // Write the rest
        self.0.write_options(writer, endian, Default::default())?;

        Ok(())
    }
}

impl<T: BinRead + BinWrite> std::ops::Deref for PreallocatedList<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: BinRead + BinWrite> std::ops::DerefMut for PreallocatedList<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/*impl<'a, T: BinRead + BinWrite> Iterator for &'a PreallocatedList<T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.0.iter().next()
    }
}*/
/*impl<'a, T: BinRead + BinWrite> IntoIterator for &'a PreallocatedList<&'a T> {
    type Item = &'a T;
    type IntoIter = std::vec::IntoIter<&'a T>;

    fn into_iter(self) -> Self::IntoIter {
        let a = &self.0.into_iter();
        a
    }
}*/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{byte_counted::ByteCounted, PascalString};
    use binrw::{binrw, io::Cursor, BinWriterExt};

    #[binrw]
    #[derive(Debug)]
    struct TestStructure {
        pub id: DynamicInteger,
        pub value: ByteCounted<PreallocatedList<PascalString>>,
    }

    #[test]
    fn read_basic() {
        let mut reader = Cursor::new(b"\x03\x05Hello\x05there\x02m8");
        let data = PreallocatedList::<PascalString>::read_be(&mut reader).unwrap();
        assert_eq!(data.0[0].0, "Hello");
        assert_eq!(data.0[1].0, "there");
        assert_eq!(data.0[2].0, "m8");
    }

    #[test]
    fn write_basic() {
        let data = PreallocatedList(vec![
            PascalString::from("Hello"),
            PascalString::from("there"),
            PascalString::from("m8"),
        ]);
        let mut writer = Cursor::new(Vec::<u8>::new());
        writer.write_be(&data).unwrap();
        assert_eq!(writer.into_inner(), b"\x03\x05Hello\x05there\x02m8");
    }

    #[test]
    fn read_complex() {
        let mut reader = Cursor::new(b"\x69\x10\x03\x05Hello\x05there\x02m8");
        let data = TestStructure::read_be(&mut reader).unwrap();
        assert_eq!(data.id.0, 0x69);
        assert_eq!(data.value[0].0, "Hello");
        assert_eq!(data.value[1].0, "there");
        assert_eq!(data.value[2].0, "m8");
    }

    #[test]
    #[should_panic]
    fn read_complex_byte_count_lower_than_should_be() {
        let mut reader = Cursor::new(b"\x69\x0F\x03\x05Hello\x05there\x02m8");
        let data = TestStructure::read_be(&mut reader).unwrap();
        assert_eq!(data.id.0, 0x69);
        assert_eq!(data.value[0].0, "Hello");
        assert_eq!(data.value[1].0, "there");
        assert_eq!(data.value[2].0, "m8");
    }

    #[test]
    #[should_panic]
    fn read_complex_byte_count_higher_than_should_be() {
        let mut reader = Cursor::new(b"\x69\x11\x03\x05Hello\x05there\x02m8");
        let data = TestStructure::read_be(&mut reader).unwrap();
        assert_eq!(data.id.0, 0x69);
        assert_eq!(data.value[0].0, "Hello");
        assert_eq!(data.value[1].0, "there");
        assert_eq!(data.value[2].0, "m8");
    }

    #[test]
    fn write_complex() {
        let data = TestStructure {
            id: DynamicInteger(0x69),
            value: ByteCounted(PreallocatedList(vec![
                PascalString::from("Hello"),
                PascalString::from("there"),
                PascalString::from("m8"),
            ])),
        };
        let mut writer = Cursor::new(Vec::<u8>::new());
        writer.write_be(&data).unwrap();
        assert_eq!(writer.into_inner(), b"\x69\x10\x03\x05Hello\x05there\x02m8");
    }
}

use crate::{
    types::DynamicInteger, util::constants::ERROR_BINRW_READ,
};
use binrw::{
    io::{Read, Seek, Write},
    BinRead, BinResult, BinWrite, BinWriterExt, Endian,
};

#[derive(Debug)]
pub struct NullTerminatedList<T: BinRead>(pub Vec<T>);

impl<T: for<'a> BinRead<Args<'a> = ()>> BinRead for NullTerminatedList<T> {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        _: Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut entries = Vec::<T>::new();

        // The first byte of the sub-section will be the event count.
        // Do not put it before the loop to gather the sub-section.
        let count = DynamicInteger::read_options(reader, endian, ()).expect(ERROR_BINRW_READ);

        for _ in 0..*count {
            let entry =
                T::read_options(reader, endian, Default::default()).expect(ERROR_BINRW_READ);
            entries.push(entry);
        }

        Ok(NullTerminatedList(entries))
    }
}

impl<T: BinRead + BinWrite + 'static> BinWrite for NullTerminatedList<T>
where
    for<'a> T: BinRead + BinWrite<Args<'a> = ()>,
    /*for<'a> T: BinRead + BinWrite,
    for<'a> <T as BinRead>::Args<'a>: Default,
    for<'a> <T as BinWrite>::Args<'a>: Default,*/
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
        writer.write_be(&self.0).unwrap();
        self.0.write_options(writer, endian, Default::default())?;

        Ok(())
    }
}

impl<T: BinRead + BinWrite> std::ops::Deref for NullTerminatedList<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: BinRead + BinWrite> std::ops::DerefMut for NullTerminatedList<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{PascalString, byte_counted::ByteCounted};

    use super::*;
    use binrw::{binrw, io::Cursor};

    #[binrw]
    #[derive(Debug)]
    struct TestStructure {
        pub id: DynamicInteger,
        pub value: ByteCounted<NullTerminatedList<PascalString>>,
    }

    #[test]
    fn read_basic() {
        let mut reader = Cursor::new(b"\x03\x05Hello\x05there\x02m8");
        let data = NullTerminatedList::<PascalString>::read_be(&mut reader).unwrap();
        assert_eq!(data.0[0].0, "Hello");
        assert_eq!(data.0[1].0, "there");
        assert_eq!(data.0[2].0, "m8");
    }

    #[test]
    fn write_basic() {
        let data = NullTerminatedList(vec![
            PascalString::from("Hello"),
            PascalString::from("there"),
            PascalString::from("m8")
        ]);
        let mut writer = Cursor::new(Vec::<u8>::new());
        writer.write_be(&data).unwrap();
        assert_eq!(
            writer.into_inner(),
            b"\x03\x05Hello\x05there\x02m8"
        );
    }
}

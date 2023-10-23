/*use crate::{structs::map::LcfMapUnitEvent, types::DynamicInteger};
use binrw::{
    io::{Cursor, Read, Seek, Write},
    BinRead, BinResult, BinWrite, BinWriterExt, Endian,
};

#[derive(Debug)]
pub struct MapEventsWrapper<T: BinRead + BinWrite>(Vec<T>);

impl<T: BinRead + BinWrite> BinRead for MapEventsWrapper<T> {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let vec = vec![];
        let byte_count = <T>::read_options(reader, endian, args)?;

        Ok(MapEventsWrapper(vec))
    }
}

impl<T: BinRead + BinWrite> BinWrite for MapEventsWrapper<T> {
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
        self.0.write_options(writer, endian, args)?;

        Ok(())
    }
}

impl<T: BinRead + BinWrite> std::ops::Deref for MapEventsWrapper<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: BinRead + BinWrite> std::ops::DerefMut for MapEventsWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}*/

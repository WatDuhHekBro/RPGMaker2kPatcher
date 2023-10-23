use crate::structs::map::LcfMapUnitHeader;
use binrw::{
    io::{Read, Seek, Write},
    BinRead, BinResult, BinWrite, Endian,
};

#[derive(Debug)]
pub struct MapMainWrapper(Vec<LcfMapUnitHeader>);

impl BinRead for MapMainWrapper {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut headers = vec![];

        loop {
            let header = <LcfMapUnitHeader>::read_options(reader, endian, ())?;

            if let LcfMapUnitHeader::End = header {
                break;
            } else {
                headers.push(header);
            }
        }

        Ok(MapMainWrapper(headers))
    }
}

impl BinWrite for MapMainWrapper {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        self.0.write_options(writer, endian, args)?;
        0u8.write_options(writer, endian, args)?;
        Ok(())
    }
}

impl std::ops::Deref for MapMainWrapper {
    type Target = Vec<LcfMapUnitHeader>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for MapMainWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

use crate::structs::map::LcfMapUnitEventHeader;
use binrw::{
    io::{Read, Seek, Write},
    BinRead, BinResult, BinWrite, Endian,
};

#[derive(Debug)]
pub struct MapEventHeadersWrapper(Vec<LcfMapUnitEventHeader>);

impl BinRead for MapEventHeadersWrapper {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut headers = vec![];

        loop {
            let header = <LcfMapUnitEventHeader>::read_options(reader, endian, ())?;

            if let LcfMapUnitEventHeader::End = header {
                break;
            } else {
                headers.push(header);
            }
        }

        Ok(MapEventHeadersWrapper(headers))
    }
}

impl BinWrite for MapEventHeadersWrapper {
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

impl std::ops::Deref for MapEventHeadersWrapper {
    type Target = Vec<LcfMapUnitEventHeader>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for MapEventHeadersWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

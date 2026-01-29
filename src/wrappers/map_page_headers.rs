// Vec<LcfMapUnitPageHeader> (null-terminated)
// -----
// Copy of map_event_headers without the Pages recursion
// TODO: Figure out a smarter way for less redundancy

use crate::structs::map::LcfMapUnitPageHeader;
use binrw::{
    io::{Read, Seek, Write},
    BinRead, BinResult, BinWrite, Endian,
};

#[derive(Debug)]
pub struct MapPageHeadersWrapper(Vec<LcfMapUnitPageHeader>);

impl BinRead for MapPageHeadersWrapper {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut headers = vec![];

        loop {
            let header = <LcfMapUnitPageHeader>::read_options(reader, endian, ())?;

            if let LcfMapUnitPageHeader::End = header {
                break;
            } else {
                headers.push(header);
            }
        }

        Ok(MapPageHeadersWrapper(headers))
    }
}

impl BinWrite for MapPageHeadersWrapper {
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

impl std::ops::Deref for MapPageHeadersWrapper {
    type Target = Vec<LcfMapUnitPageHeader>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for MapPageHeadersWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

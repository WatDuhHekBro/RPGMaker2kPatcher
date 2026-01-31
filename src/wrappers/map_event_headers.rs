// Vec<LcfMapUnitEventHeader> (null-terminated)

use crate::{
    structs::map::LcfMapUnitEventHeader,
    wrappers::{MapPageHeadersWrapper, MapPagesWrapper},
    ERROR_BINRW_READ,
};
use binrw::{
    io::{Read, Seek, Write},
    BinRead, BinResult, BinWrite, Endian,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct MapEventHeadersWrapper(pub Vec<LcfMapUnitEventHeader>);

impl MapEventHeadersWrapper {
    pub fn get_pages(&self) -> Option<&MapPagesWrapper> {
        for entry in &self.0 {
            if let LcfMapUnitEventHeader::Pages(pages) = entry {
                return Some(&pages);
            }
        }

        None
    }
    pub fn get_pages_mut(&mut self) -> Option<&mut MapPagesWrapper> {
        for entry in &mut self.0 {
            if let LcfMapUnitEventHeader::Pages(ref mut pages) = entry {
                return Some(pages);
            }
        }

        None
    }

    pub fn get_page(&self, id: i32) -> Option<&MapPageHeadersWrapper> {
        self.get_pages().and_then(|pages| pages.get_page(id))
    }

    pub fn get_page_mut(&mut self, id: i32) -> Option<&mut MapPageHeadersWrapper> {
        self.get_pages_mut().and_then(|pages| pages.get_page_mut(id))
    }
}

impl BinRead for MapEventHeadersWrapper {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut headers = vec![];

        loop {
            let header =
                LcfMapUnitEventHeader::read_options(reader, endian, ()).expect(ERROR_BINRW_READ);

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

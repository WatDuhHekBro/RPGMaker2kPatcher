// Wrapper: Byte Count (DynamicInteger), # of Pages Count (DynamicInteger), Vec<LcfMapUnitPage>
// -----
// Copy of map_events without the Events recursion
// TODO: Figure out a smarter way for less redundancy

use crate::{
    structs::map::LcfMapUnitPage, types::DynamicInteger, wrappers::MapPageHeadersWrapper,
    ERROR_BINRW_READ,
};
use binrw::{
    io::{Cursor, Read, Seek, Write},
    BinRead, BinResult, BinWrite, BinWriterExt, Endian,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct MapPagesWrapper(pub Vec<LcfMapUnitPage>);

impl MapPagesWrapper {
    pub fn get_page(&self, id: i32) -> Option<&MapPageHeadersWrapper> {
        for page in &self.0 {
            if page.id.0 == id {
                return Some(&page.headers);
            }
        }

        None
    }
    pub fn get_page_mut(&mut self, id: i32) -> Option<&mut MapPageHeadersWrapper> {
        for page in &mut self.0 {
            if page.id.0 == id {
                return Some(&mut page.headers);
            }
        }

        None
    }
}

impl BinRead for MapPagesWrapper {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<Self> {
        // Unless you can read the amount of bytes of a struct, just operate on a separate pool of bytes
        let mut bytes = Vec::<u8>::new();
        let byte_count = DynamicInteger::read_options(reader, endian, ()).expect(ERROR_BINRW_READ);

        for _ in 0..*byte_count {
            let byte = <u8>::read_options(reader, endian, ()).expect(ERROR_BINRW_READ);
            bytes.push(byte);
        }

        // This entire section will now operate on this section of bytes
        let mut reader = Cursor::new(&bytes);
        let mut pages = Vec::<LcfMapUnitPage>::new();

        // The first byte of the sub-section will be the event count.
        // Do not put it before the loop to gather the sub-section.
        let page_count =
            DynamicInteger::read_options(&mut reader, endian, ()).expect(ERROR_BINRW_READ);

        for _ in 0..*page_count {
            let page = LcfMapUnitPage::read(&mut reader).unwrap();
            pages.push(page);
        }

        Ok(MapPagesWrapper(pages))
    }
}

impl BinWrite for MapPagesWrapper {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        let mut subsection_writer = Cursor::new(Vec::<u8>::new());
        // Write back the event count before writing the rest of the bytes
        DynamicInteger(self.0.len().try_into().unwrap()).write_options(
            &mut subsection_writer,
            endian,
            args,
        )?;
        // Write the rest of the bytes
        subsection_writer.write_be(&self.0).unwrap();
        let bytes = subsection_writer.into_inner();
        //println!("{:02X?}", bytes);

        // Then write the sub-section into the main section of bytes
        DynamicInteger(bytes.len().try_into().unwrap()).write_options(writer, endian, args)?;
        bytes.write_options(writer, endian, args)?;

        Ok(())
    }
}

impl std::ops::Deref for MapPagesWrapper {
    type Target = Vec<LcfMapUnitPage>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for MapPagesWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

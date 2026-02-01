// Vec<LcfMapUnitPageHeader> (null-terminated)
// -----
// Copy of map_event_headers without the Pages recursion
// TODO: Figure out a smarter way for less redundancy

use crate::{
    structs::map::{LcfMapUnitPageHeader, LcfMapUnitPageHeaderGeneric},
    types::{DynamicInteger, PascalString, U8Array},
    util::constants::*,
    wrappers::MapCommandsWrapper,
};
use binrw::{
    io::{Read, Seek, Write},
    BinRead, BinResult, BinWrite, Endian,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct MapPageHeadersWrapper(pub Vec<LcfMapUnitPageHeader>);

impl MapPageHeadersWrapper {
    pub fn get_commands(&self) -> Option<&MapCommandsWrapper> {
        for entry in &self.0 {
            if let LcfMapUnitPageHeader::Commands(commands) = entry {
                return Some(&commands);
            }
        }

        None
    }
    pub fn get_commands_mut(&mut self) -> Option<&mut MapCommandsWrapper> {
        for entry in &mut self.0 {
            if let LcfMapUnitPageHeader::Commands(ref mut commands) = entry {
                return Some(commands);
            }
        }

        None
    }
}

impl BinRead for MapPageHeadersWrapper {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut headers = vec![];

        loop {
            // For some reason, "#[brw(magic = 51u8)]" doesn't seem to reroute the page header, so it looks like you'll need to sort this out.
            //let header = LcfMapUnitPageHeader::read_options(reader, endian, ()).expect(ERROR_BINRW_READ);
            let id = DynamicInteger::read_options(reader, endian, ()).expect(ERROR_BINRW_READ);

            if id.0 == 0 {
                break;
            } else if id.0 == 21 {
                headers.push(LcfMapUnitPageHeader::Name(
                    PascalString::read_options(reader, endian, ()).expect(ERROR_BINRW_READ),
                ));
            }
            // 0x33 contains redundant byte count, immediately followed by 0x34 which contains the commands
            // Wrapper: Byte Count Length (DynamicInteger) (DISCARD), Byte Count (DynamicInteger) (DISCARD), 0x34 (52)
            // Byte Count (DynamicInteger), # of Pages Count (DynamicInteger), Vec<LcfMapUnitCommand> (null-terminated by a 4-set of zeroes)
            else if id.0 == 51 {
                headers.push(LcfMapUnitPageHeader::Commands(
                    MapCommandsWrapper::read_options(reader, endian, ()).expect(ERROR_BINRW_READ),
                ));
            } else {
                headers.push(LcfMapUnitPageHeader::Generic(LcfMapUnitPageHeaderGeneric {
                    id,
                    value: U8Array::read_options(reader, endian, ()).expect(ERROR_BINRW_READ),
                }));
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

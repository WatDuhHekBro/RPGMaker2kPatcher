// Wrapper: Vec<LcfMapUnitHeader> (null-terminated)

use crate::{structs::map::LcfMapUnitHeader, util::constants::*, wrappers::MapEventsWrapper};
use binrw::{
    io::{Read, Seek, Write},
    BinRead, BinResult, BinWrite, Endian,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct MapMainWrapper(pub Vec<LcfMapUnitHeader>);

impl MapMainWrapper {
    pub fn get_events(&self) -> Option<&MapEventsWrapper> {
        for header in &self.0 {
            if let LcfMapUnitHeader::Events(events) = header {
                return Some(events);
            }
        }

        None
    }
    pub fn get_events_mut(&mut self) -> Option<&mut MapEventsWrapper> {
        for header in &mut self.0 {
            if let LcfMapUnitHeader::Events(events) = header {
                return Some(events);
            }
        }

        None
    }
}

impl BinRead for MapMainWrapper {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut headers = vec![];

        loop {
            let header =
                LcfMapUnitHeader::read_options(reader, endian, ()).expect(ERROR_BINRW_READ);

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

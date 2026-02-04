use crate::{
    types::{DynamicInteger, DynamicIntegerArray, NullTerminatedList, PascalString, U8Array},
    util::constants::ERROR_BINRW_READ,
};
use binrw::{
    binrw,
    io::{Read, Seek, Write},
    BinRead, BinResult, BinWrite, Endian,
};
use serde::{Deserialize, Serialize};

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
pub struct ListEntry<T>
where
    for<'a> T: BinRead<Args<'a> = ()> + BinWrite<Args<'a> = ()> + 'static + std::fmt::Debug,
{
    pub id: DynamicInteger,
    pub headers: NullTerminatedList<T>,
}

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
pub struct ListEntryHeaderGeneric {
    pub id: DynamicInteger,
    pub value: U8Array,
}

// NOTE: You cannot use NullTerminatedList<LcfCommand>, as this is a special case of terminating via a 4-set of zeroes, not 0x00.

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
#[brw(big)]
// If I remember correctly, the indent is mostly just for viewing it in an editor (EasyRPG Editor or the official RPGMaker2k)
pub struct LcfCommand {
    pub code: DynamicInteger,            // 1st
    pub indent: DynamicInteger,          // 2nd
    pub text: PascalString,              // 3rd
    pub parameters: DynamicIntegerArray, // 4th
}

impl LcfCommand {
    pub fn is_terminating(&self) -> bool {
        *self.code == 0 && *self.indent == 0 && self.text.is_empty() && self.parameters.is_empty()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LcfCommandList(pub Vec<LcfCommand>); // (null-terminated by a 4-set of zeroes)

impl BinRead for LcfCommandList {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        _: Endian,
        _: Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut commands: Vec<LcfCommand> = Vec::new();

        loop {
            let command = LcfCommand::read_be(reader).expect(ERROR_BINRW_READ);

            if command.is_terminating() {
                break;
            } else {
                commands.push(command);
            }
        }

        Ok(LcfCommandList(commands))
    }
}

impl BinWrite for LcfCommandList {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        // Write the initial command bytes
        self.0.write_options(writer, endian, args)?;
        // Add a null-terminating command set
        0u32.write_options(writer, endian, args)?;

        Ok(())
    }
}

impl std::ops::Deref for LcfCommandList {
    type Target = Vec<LcfCommand>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for LcfCommandList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_command_should_terminate() {
        let command = LcfCommand {
            code: DynamicInteger(0),
            indent: DynamicInteger(0),
            text: PascalString::from(""),
            parameters: DynamicIntegerArray(vec![]),
        };
        assert_eq!(command.is_terminating(), true);
    }
}

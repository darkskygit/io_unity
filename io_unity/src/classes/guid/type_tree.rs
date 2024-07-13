use super::{Guid, GuidObject};
use crate::error::ReadResult;
use crate::type_tree::convert::TryCastFrom;
use binrw::binrw;
use num_enum::TryFromPrimitive;

impl GuidObject for Guid<'_> {
    fn get_guid(&self) -> ReadResult<String> {
        let guid_data = self.get_guid_data()?;
        Ok(format!(
            "{:08x}-{:04x}-{:04x}-{:04x}-{:04x}{:08x}",
            guid_data[0],
            guid_data[1] >> 16,
            guid_data[1] & 0xFFFF,
            guid_data[2] >> 16,
            guid_data[2] & 0xFFFF,
            guid_data[3]
        ))
    }
}

impl Guid<'_> {
    fn get_guid_data(&self) -> ReadResult<[u32; 4]> {
        let first = u32::try_cast_from(self.inner, "/Base/data[0]")?;
        let second = u32::try_cast_from(self.inner, "/Base/data[1]")?;
        let third = u32::try_cast_from(self.inner, "/Base/data[2]")?;
        let fourth = u32::try_cast_from(self.inner, "/Base/data[3]")?;
        Ok([first, second, third, fourth])
    }
}

#[binrw]
#[brw(repr = u32)]
#[derive(Debug, Eq, PartialEq, TryFromPrimitive, Clone)]
#[repr(u32)]
pub enum AudioCompressionFormat {
    PCM = 0,
    Vorbis = 1,
    ADPCM = 2,
    MP3 = 3,
    VAG = 4,
    HEVAG = 5,
    XMA = 6,
    AAC = 7,
    GCADPCM = 8,
    ATRAC9 = 9,
}

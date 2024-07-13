use super::{KeyValuePair, KeyValuePairObject, TypeTreeObjectRef};
use crate::error::ReadResult;
use crate::type_tree::convert::TryCastFrom;
use binrw::binrw;
use num_enum::TryFromPrimitive;

impl KeyValuePairObject for KeyValuePair<'_> {
    fn get_pair(&self) -> ReadResult<(TypeTreeObjectRef, TypeTreeObjectRef)> {
        let key = self.get_key()?;
        let value = self.get_value()?;
        Ok((key, value))
    }
}

impl KeyValuePair<'_> {
    fn get_key(&self) -> ReadResult<TypeTreeObjectRef> {
        TypeTreeObjectRef::try_cast_from(self.inner, "/Base/first")
    }

    fn get_value(&self) -> ReadResult<TypeTreeObjectRef> {
        TypeTreeObjectRef::try_cast_from(self.inner, "/Base/second")
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

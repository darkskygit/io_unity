use std::{collections::HashMap, sync::Arc};

use binrw::Endian;

use super::{ArrayField, ArrayFieldValue, DataOffset, Field, FieldValue, TypeField, TypeTreeObject, TypeTreeObjectRef};
use crate::type_tree::convert::TryCastFrom;

#[derive(Debug)]
struct FixtureTypeField {
  name: String,
  type_name: String,
  byte_size: i32,
}

impl TypeField for FixtureTypeField {
  fn get_version(&self) -> u16 {
    1
  }

  fn get_level(&self) -> u8 {
    0
  }

  fn is_array(&self) -> bool {
    false
  }

  fn get_byte_size(&self) -> i32 {
    self.byte_size
  }

  fn get_index(&self) -> i32 {
    0
  }

  fn get_meta_flag(&self) -> i32 {
    0
  }

  fn is_align(&self) -> bool {
    false
  }

  fn get_ref_type_hash(&self) -> Option<u64> {
    None
  }

  fn get_type(&self) -> &String {
    &self.type_name
  }

  fn get_name(&self) -> &String {
    &self.name
  }
}

fn type_field(name: &str, type_name: &str, byte_size: i32) -> Arc<Box<dyn TypeField + Send + Sync>> {
  Arc::new(Box::new(FixtureTypeField {
    name: name.to_owned(),
    type_name: type_name.to_owned(),
    byte_size,
  }))
}

fn data_field(name: &str, type_name: &str, byte_size: i32, offset: u64) -> Field {
  Field {
    field_type: type_field(name, type_name, byte_size),
    data: FieldValue::DataOffset(DataOffset::AbsDataOffset(offset)),
  }
}

fn string_field(name: &str, size_offset: u64, data_offset: u64) -> Field {
  let array = Field {
    field_type: type_field("Array", "Array", -1),
    data: FieldValue::Array(Box::new(ArrayField {
      array_size: data_field("size", "int", 4, size_offset),
      item_type_fields: vec![],
      item_field: Some(data_field("data", "char", 1, data_offset)),
      item_field_size: Some(1),
      data: ArrayFieldValue::DataOffset(DataOffset::AbsDataOffset(data_offset)),
    })),
  };

  Field {
    field_type: type_field(name, "string", -1),
    data: FieldValue::Fields(HashMap::from([("Array".to_owned(), array)])),
  }
}

fn text_asset_fixture() -> TypeTreeObjectRef {
  const NAME: &[u8] = b"anonymous";
  const SCRIPT: &[u8] = b"fixture payload";

  let mut data = Vec::new();
  data.extend_from_slice(&(NAME.len() as i32).to_le_bytes());
  data.extend_from_slice(NAME);
  let script_size_offset = data.len() as u64;
  data.extend_from_slice(&(SCRIPT.len() as i32).to_le_bytes());
  let script_data_offset = data.len() as u64;
  data.extend_from_slice(SCRIPT);

  let fields = HashMap::from([
    ("m_Name".to_owned(), string_field("m_Name", 0, 4)),
    (
      "m_Script".to_owned(),
      string_field("m_Script", script_size_offset, script_data_offset),
    ),
  ]);

  TypeTreeObject {
    endian: Endian::Little,
    class_id: 49,
    serialized_file_id: 7,
    path_id: 11,
    data_layout: Field {
      field_type: type_field("Base", "TextAsset", -1),
      data: FieldValue::Fields(fields),
    },
    data_buff: data,
    external_data: None,
  }
  .into()
}

#[test]
fn reads_text_asset_fields_by_unity_path() {
  let object = text_asset_fixture();

  assert_eq!(String::try_cast_from(&object, "/Base/m_Name").unwrap(), "anonymous");
  assert_eq!(String::try_cast_from(&object, "/Base/m_Script").unwrap(), "fixture payload");
  assert_eq!(object.get_class_id(), 49);
  assert_eq!(object.get_serialized_file_id(), 7);
  assert_eq!(object.get_path_id(), 11);
}

#[test]
fn exposes_binary_text_asset_payload_without_copying() {
  let object = text_asset_fixture();
  let object = object.inner().read().unwrap();

  assert_eq!(object.try_as_slice("/Base/m_Script/Array").unwrap(), b"fixture payload");
}

#[cfg(feature = "external-class-handle")]
#[test]
fn named_object_reads_the_same_name_path_used_by_consumers() {
  use crate::classes::named_object::{NamedObject, NamedObjectObject};

  let object = text_asset_fixture();
  assert_eq!(NamedObject::new(&object).get_name().unwrap(), "anonymous");
}

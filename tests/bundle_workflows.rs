use std::io::{Cursor, Read, Seek, SeekFrom};

use io_unity_next::{unity_asset_view::UnityAssetViewer, unityfs::UnityFS};

const CAB_PATH: &str = "CAB-00000000000000000000000000000000";

fn push_null_string(buffer: &mut Vec<u8>, value: &str) {
  buffer.extend_from_slice(value.as_bytes());
  buffer.push(0);
}

fn synthetic_serialized_file() -> Vec<u8> {
  let mut content = Vec::new();
  push_null_string(&mut content, "2020.3.0f1");
  content.extend_from_slice(&13_i32.to_le_bytes());
  content.push(0);
  content.extend_from_slice(&0_u32.to_le_bytes());
  content.extend_from_slice(&0_i32.to_le_bytes());
  content.extend_from_slice(&0_i32.to_le_bytes());
  content.extend_from_slice(&0_i32.to_le_bytes());
  content.push(0);

  let file_size = 20 + content.len();
  let mut file = Vec::with_capacity(file_size);
  file.extend_from_slice(&(content.len() as u32).to_be_bytes());
  file.extend_from_slice(&(file_size as u32).to_be_bytes());
  file.extend_from_slice(&17_u32.to_be_bytes());
  file.extend_from_slice(&(file_size as u32).to_be_bytes());
  file.push(0);
  file.extend_from_slice(&[0; 3]);
  file.extend_from_slice(&content);
  file
}

fn synthetic_v22_serialized_file_with_negative_ref_type(script_type_index: i16) -> Vec<u8> {
  let mut content = Vec::new();
  push_null_string(&mut content, "2022.3.0f1");
  content.extend_from_slice(&13_i32.to_le_bytes());
  content.push(0);
  content.extend_from_slice(&0_u32.to_le_bytes());
  content.extend_from_slice(&0_i32.to_le_bytes());
  content.extend_from_slice(&0_i32.to_le_bytes());
  content.extend_from_slice(&0_i32.to_le_bytes());
  content.extend_from_slice(&1_i32.to_le_bytes());
  content.extend_from_slice(&(-1_i32).to_le_bytes());
  content.push(0);
  content.extend_from_slice(&script_type_index.to_le_bytes());
  if script_type_index >= 0 {
    content.extend_from_slice(&[0xab; 16]);
  }
  content.extend_from_slice(&[0xcd; 16]);
  push_null_string(&mut content, "SyntheticRef");
  push_null_string(&mut content, "Tests");
  push_null_string(&mut content, "Synthetic.dll");
  push_null_string(&mut content, "fixture-complete");

  let header_size = 48_u64;
  let file_size = header_size + content.len() as u64;
  let mut file = Vec::with_capacity(file_size as usize);
  file.extend_from_slice(&(content.len() as u32).to_be_bytes());
  file.extend_from_slice(&(file_size as u32).to_be_bytes());
  file.extend_from_slice(&22_u32.to_be_bytes());
  file.extend_from_slice(&(file_size as u32).to_be_bytes());
  file.push(0);
  file.extend_from_slice(&[0; 3]);
  file.extend_from_slice(&(content.len() as u32).to_be_bytes());
  file.extend_from_slice(&file_size.to_be_bytes());
  file.extend_from_slice(&file_size.to_be_bytes());
  file.extend_from_slice(&0_u64.to_be_bytes());
  file.extend_from_slice(&content);
  file
}

fn synthetic_bundle() -> (Vec<u8>, Vec<u8>) {
  let serialized_file = synthetic_serialized_file();

  let mut blocks_info = vec![0; 16];
  blocks_info.extend_from_slice(&1_u32.to_be_bytes());
  blocks_info.extend_from_slice(&(serialized_file.len() as u32).to_be_bytes());
  blocks_info.extend_from_slice(&(serialized_file.len() as i32).to_be_bytes());
  blocks_info.extend_from_slice(&0_u16.to_be_bytes());
  blocks_info.extend_from_slice(&1_u32.to_be_bytes());
  blocks_info.extend_from_slice(&0_i64.to_be_bytes());
  blocks_info.extend_from_slice(&(serialized_file.len() as i64).to_be_bytes());
  blocks_info.extend_from_slice(&0_u32.to_be_bytes());
  push_null_string(&mut blocks_info, CAB_PATH);

  let mut bundle = Vec::new();
  bundle.extend_from_slice(b"UnityFS\0");
  bundle.extend_from_slice(&6_u32.to_be_bytes());
  push_null_string(&mut bundle, "2020.3.0f1");
  push_null_string(&mut bundle, "2020.3.0f1");
  let size_offset = bundle.len();
  bundle.extend_from_slice(&0_i64.to_be_bytes());
  bundle.extend_from_slice(&(blocks_info.len() as u32).to_be_bytes());
  bundle.extend_from_slice(&(blocks_info.len() as u32).to_be_bytes());
  bundle.extend_from_slice(&0_u32.to_be_bytes());
  bundle.extend_from_slice(&blocks_info);
  bundle.extend_from_slice(&serialized_file);
  let bundle_size = bundle.len() as i64;
  bundle[size_offset..size_offset + 8].copy_from_slice(&bundle_size.to_be_bytes());

  (bundle, serialized_file)
}

#[test]
fn reads_anonymous_cab_from_uncompressed_unityfs_bundle() {
  let (bundle, serialized_file) = synthetic_bundle();
  let unity_fs = UnityFS::read(Box::new(Cursor::new(bundle)), None).unwrap();

  assert_eq!(unity_fs.get_cab_path(), vec![CAB_PATH]);
  assert_eq!(unity_fs.get_file_paths(), vec![CAB_PATH]);
  assert_eq!(unity_fs.get_file_data_by_path(&CAB_PATH.to_owned()).unwrap(), serialized_file);

  let mut node = unity_fs.get_file_reader_by_path(&CAB_PATH.to_owned()).unwrap();
  node.seek(SeekFrom::Start(8)).unwrap();
  let mut version = [0; 4];
  node.read_exact(&mut version).unwrap();
  assert_eq!(u32::from_be_bytes(version), 17);
}

#[test]
fn asset_viewer_registers_cab_and_serialized_file() {
  let (bundle, _) = synthetic_bundle();
  let mut viewer = UnityAssetViewer::new();

  assert_eq!(viewer.add_bundle_file(Box::new(Cursor::new(bundle)), None).unwrap(), 0);
  assert_eq!(viewer.cab_maps.get(CAB_PATH), Some(&0));
  assert!(viewer.serialized_file_map.get(&0).unwrap().get_object_map().is_empty());
}

#[test]
fn reads_v22_negative_ref_type_script_identity() {
  let with_script = synthetic_v22_serialized_file_with_negative_ref_type(0);
  let parsed = io_unity_next::serialized_file::SerializedFile::read(Box::new(Cursor::new(with_script)), 0, None).unwrap();
  assert!(parsed.get_object_map().is_empty());

  let without_script = synthetic_v22_serialized_file_with_negative_ref_type(-1);
  let parsed = io_unity_next::serialized_file::SerializedFile::read(Box::new(Cursor::new(without_script)), 0, None).unwrap();
  assert!(parsed.get_object_map().is_empty());
}

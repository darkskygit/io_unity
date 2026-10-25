extern crate io_unity_next;

use std::{
  fs::File,
  io::{Cursor, Read},
  path::PathBuf,
};

use io_unity_next::unityfs::UnityFS;
use serde::Serialize;
use serde_json::json;

#[derive(Serialize)]
struct BlockEntry {
  index: usize,
  compressed_size: i32,
  uncompressed_size: u32,
  compression: u8,
  compression_name: &'static str,
}

#[derive(Serialize)]
struct NodeEntry {
  path: String,
  extracted_size: usize,
}

#[derive(Serialize)]
struct Survey {
  file: String,
  file_size: u64,
  header_version: Option<u32>,
  unity_version: Option<String>,
  unity_revision: Option<String>,
  header_size_field: Option<i64>,
  archive_flags_raw: Option<u32>,
  archive_compression: Option<u8>,
  archive_compression_name: Option<&'static str>,
  storage_blocks: Vec<BlockEntry>,
  nodes: Vec<NodeEntry>,
  full_unpack_ok: bool,
  errors: Vec<String>,
}

fn compression_name(code: u8) -> &'static str {
  match code {
    0 => "None",
    1 => "Lzma",
    2 => "Lz4",
    3 => "Lz4HC",
    4 => "Lzham",
    _ => "Unknown",
  }
}

fn parse_raw_header(bytes: &[u8]) -> Result<(u32, String, String, i64, u32), String> {
  // Standard UnityFS header: magic(8) + u32be version + unity_version NullString
  // + unity_revision NullString + i64be size + u32be compressed_blocks_info_size
  // + u32be uncompressed_blocks_info_size + u32 flags (bitfield, LE).
  if bytes.len() < 8 || &bytes[0..8] != b"UnityFS\0" {
    return Err(format!("not a UnityFS header, first 64 bytes hex: {}", hex64(bytes)));
  }
  let mut pos = 8;
  let take_u32be = |pos: &mut usize| -> u32 {
    let v = u32::from_be_bytes(bytes[*pos..*pos + 4].try_into().unwrap());
    *pos += 4;
    v
  };
  let version = take_u32be(&mut pos);
  let take_null_string = |pos: &mut usize| -> String {
    let start = *pos;
    while *pos < bytes.len() && bytes[*pos] != 0 {
      *pos += 1;
    }
    let s = String::from_utf8_lossy(&bytes[start..*pos]).to_string();
    *pos += 1;
    s
  };
  let unity_version = take_null_string(&mut pos);
  let unity_revision = take_null_string(&mut pos);
  let size = i64::from_be_bytes(bytes[pos..pos + 8].try_into().unwrap());
  pos += 8;
  pos += 8; // compressed/uncompressed blocks info sizes, not needed here
  let flags_raw = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap());
  Ok((version, unity_version, unity_revision, size, flags_raw))
}

fn hex64(bytes: &[u8]) -> String {
  bytes.iter().take(64).map(|b| format!("{b:02x}")).collect::<Vec<_>>().join("")
}

fn survey_file(path: &PathBuf) -> Survey {
  let mut file = match File::open(path) {
    Ok(f) => f,
    Err(e) => {
      return Survey {
        file: path.display().to_string(),
        file_size: 0,
        header_version: None,
        unity_version: None,
        unity_revision: None,
        header_size_field: None,
        archive_flags_raw: None,
        archive_compression: None,
        archive_compression_name: None,
        storage_blocks: vec![],
        nodes: vec![],
        full_unpack_ok: false,
        errors: vec![format!("open: {e}")],
      };
    }
  };
  let mut bytes = Vec::new();
  file.read_to_end(&mut bytes).unwrap();
  let file_size = bytes.len() as u64;

  let mut errors = Vec::new();
  let (header_version, unity_version, unity_revision, header_size_field, archive_flags_raw) = match parse_raw_header(&bytes) {
    Ok((v, uv, ur, sz, fl)) => (Some(v), Some(uv), Some(ur), Some(sz), Some(fl)),
    Err(e) => {
      errors.push(e);
      (None, None, None, None, None)
    }
  };

  let mut storage_blocks = Vec::new();
  let mut nodes = Vec::new();
  let mut full_unpack_ok = false;
  let mut archive_compression = None;
  let mut archive_compression_name = None;

  if header_version.is_some() {
    match UnityFS::read(Box::new(Cursor::new(bytes)), None) {
      Ok(ufs) => {
        let (version, archive_code, blocks) = ufs.compression_survey();
        if header_version != Some(version) {
          errors.push(format!("header version mismatch: raw={header_version:?} parsed={version}"));
        }
        archive_compression = Some(archive_code);
        archive_compression_name = Some(compression_name(archive_code));
        storage_blocks = blocks
          .iter()
          .enumerate()
          .map(|(i, (cs, us, code))| BlockEntry {
            index: i,
            compressed_size: *cs,
            uncompressed_size: *us,
            compression: *code,
            compression_name: compression_name(*code),
          })
          .collect();
        let mut all_ok = true;
        for path in ufs.get_file_paths() {
          match ufs.get_file_data_by_path(&path) {
            Ok(data) => nodes.push(NodeEntry {
              path,
              extracted_size: data.len(),
            }),
            Err(e) => {
              all_ok = false;
              errors.push(format!("extract {path}: {e}"));
            }
          }
        }
        full_unpack_ok = all_ok && !nodes.is_empty();
      }
      Err(e) => errors.push(format!("UnityFS::read: {e}")),
    }
  }

  Survey {
    file: path.display().to_string(),
    file_size,
    header_version,
    unity_version,
    unity_revision,
    header_size_field,
    archive_flags_raw,
    archive_compression,
    archive_compression_name,
    storage_blocks,
    nodes,
    full_unpack_ok,
    errors,
  }
}

fn main() {
  let mut results = Vec::new();
  for arg in std::env::args().skip(1) {
    let path = PathBuf::from(arg);
    results.push(survey_file(&path));
  }
  println!("{}", json!({ "bundles": results }));
}

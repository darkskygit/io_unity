use image::Rgba;

use super::{Texture2DObject, TextureFormat};
use crate::{error::ReadResult, unity_asset_view::UnityAssetViewer};

struct FixtureTexture {
  width: u64,
  height: u64,
  format: TextureFormat,
  data: Vec<u8>,
}

impl Texture2DObject for FixtureTexture {
  fn get_width(&self) -> ReadResult<u64> {
    Ok(self.width)
  }

  fn get_height(&self) -> ReadResult<u64> {
    Ok(self.height)
  }

  fn get_texture_format(&self) -> ReadResult<TextureFormat> {
    Ok(self.format.clone())
  }

  fn get_image_data(&self, _viewer: &UnityAssetViewer) -> ReadResult<Vec<u8>> {
    Ok(self.data.clone())
  }
}

#[test]
fn decodes_rgba32_texture_data() {
  let texture = FixtureTexture {
    width: 2,
    height: 1,
    format: TextureFormat::RGBA32,
    data: vec![255, 0, 0, 255, 0, 128, 255, 64],
  };

  let image = texture.get_image(&UnityAssetViewer::new()).unwrap().to_rgba8();
  assert_eq!(image.dimensions(), (2, 1));
  assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
  assert_eq!(*image.get_pixel(1, 0), Rgba([0, 128, 255, 64]));
}

#[test]
fn expands_argb4444_channels_to_rgba8() {
  let texture = FixtureTexture {
    width: 1,
    height: 1,
    format: TextureFormat::ARGB4444,
    data: 0xF123_u16.to_le_bytes().to_vec(),
  };

  let image = texture.get_image(&UnityAssetViewer::new()).unwrap().to_rgba8();
  assert_eq!(*image.get_pixel(0, 0), Rgba([0x11, 0x22, 0x33, 0xFF]));
}

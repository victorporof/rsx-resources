/*
Copyright 2016 Mozilla
Licensed under the Apache License, Version 2.0 (the "License"); you may not use
this file except in compliance with the License. You may obtain a copy of the
License at http://www.apache.org/licenses/LICENSE-2.0
Unless required by applicable law or agreed to in writing, software distributed
under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND, either express or implied. See the License for the
specific language governing permissions and limitations under the License.
*/

use std::sync::Arc;

#[cfg(not(feature = "image-dummy-decode"))]
use image::{load_from_memory_with_format, DynamicImage, Rgba};
#[cfg(not(feature = "image-dummy-decode"))]
use imageproc::map::map_colors;
use rsx_shared::traits::TEncodedImage;

use error::Result;
use types::{ImageEncodingFormat, ImagePixelFormat, ImageResourceData};

#[derive(Debug, PartialEq)]
pub struct DecodedImage {
    pub format: ImagePixelFormat,
    pub size: (u32, u32),
    pub pixels: Arc<Vec<u8>>
}

impl DecodedImage {
    pub fn from_raw_parts(format: ImagePixelFormat, size: (u32, u32), pixels: Arc<Vec<u8>>) -> Result<DecodedImage> {
        Ok(DecodedImage {
            format,
            size,
            pixels
        })
    }

    #[cfg(feature = "image-dummy-decode")]
    pub fn from_encoded_image<E>(encoded: &E) -> Result<DecodedImage>
    where
        E: TEncodedImage
    {
        let format = ImagePixelFormat::RGBA(0);
        let size = encoded.size_info().unwrap_or_default();
        let pixels = Arc::default();
        Self::from_raw_parts(format, size, pixels)
    }

    #[cfg(not(feature = "image-dummy-decode"))]
    pub fn from_encoded_image<E>(encoded: &E) -> Result<DecodedImage>
    where
        E: TEncodedImage
    {
        Self::load_from_memory(encoded.format().unwrap(), encoded.bytes().unwrap())
    }

    #[cfg(not(feature = "image-dummy-decode"))]
    fn load_from_memory(format: ImageEncodingFormat, bytes: &[u8]) -> Result<DecodedImage> {
        Ok(match load_from_memory_with_format(bytes, format.into())? {
            DynamicImage::ImageLuma8(data) => DecodedImage {
                format: ImagePixelFormat::Gray(8),
                size: data.dimensions(),
                pixels: Arc::new(data.into_raw())
            },
            DynamicImage::ImageRgba8(data) => {
                let (data, format) = if cfg!(feature = "image-rgb-to-bgr") {
                    let data = map_colors(&data, |p| Rgba([p[2], p[1], p[0], p[3]]));
                    (data, ImagePixelFormat::BGRA(8))
                } else {
                    (data, ImagePixelFormat::RGBA(8))
                };
                DecodedImage {
                    format,
                    size: data.dimensions(),
                    pixels: Arc::new(data.into_raw())
                }
            }
            image => {
                let (data, format) = if cfg!(feature = "image-rgb-to-bgr") {
                    let data = map_colors(&image.to_rgba(), |p| Rgba([p[2], p[1], p[0], p[3]]));
                    (data, ImagePixelFormat::BGRA(8))
                } else {
                    let data = image.to_rgba();
                    (data, ImagePixelFormat::RGBA(8))
                };
                DecodedImage {
                    format,
                    size: data.dimensions(),
                    pixels: Arc::new(data.into_raw())
                }
            }
        })
    }

    pub fn info(&self) -> ImageResourceData {
        ImageResourceData {
            format: self.format,
            size: self.size,
            pixels: &self.pixels
        }
    }
}

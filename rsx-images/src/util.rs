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

use std::io::Cursor;

use image::ImageDecoder;
use image::bmp::BMPDecoder;
use image::gif::Decoder as GIFDecoder;
use image::hdr::HDRDecoder;
use image::ico::ICODecoder;
use image::jpeg::JPEGDecoder;
use image::png::PNGDecoder;
use image::pnm::PNMDecoder;
use image::tga::TGADecoder;
use image::tiff::TIFFDecoder;
use image::webp::WebpDecoder;

use error::Result;
use types::ImageEncodingFormat;

pub fn get_dimensions(format: ImageEncodingFormat, bytes: &[u8]) -> Result<(u32, u32)> {
    Ok(match format {
        ImageEncodingFormat::PNG => {
            let mut decoder = PNGDecoder::new(bytes);
            decoder.dimensions()?
        }
        ImageEncodingFormat::JPEG => {
            let mut decoder = JPEGDecoder::new(bytes);
            decoder.dimensions()?
        }
        ImageEncodingFormat::GIF => {
            let mut decoder = GIFDecoder::new(bytes);
            decoder.dimensions()?
        }
        ImageEncodingFormat::WEBP => {
            let mut decoder = WebpDecoder::new(bytes);
            decoder.dimensions()?
        }
        ImageEncodingFormat::PNM => {
            let mut decoder = PNMDecoder::new(bytes)?;
            decoder.dimensions()?
        }
        ImageEncodingFormat::TIFF => {
            let mut decoder = TIFFDecoder::new(Cursor::new(bytes))?;
            decoder.dimensions()?
        }
        ImageEncodingFormat::TGA => {
            let mut decoder = TGADecoder::new(Cursor::new(bytes));
            decoder.dimensions()?
        }
        ImageEncodingFormat::BMP => {
            let mut decoder = BMPDecoder::new(Cursor::new(bytes));
            decoder.dimensions()?
        }
        ImageEncodingFormat::ICO => {
            let mut decoder = ICODecoder::new(Cursor::new(bytes))?;
            decoder.dimensions()?
        }
        ImageEncodingFormat::HDR => {
            let metadata = HDRDecoder::new(bytes)?.metadata();
            (metadata.width, metadata.height)
        }
    })
}

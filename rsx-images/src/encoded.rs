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

use std::rc::Rc;

use image;

use base64_util;
use rsx_shared::traits::TEncodedImage;

use error::{ImageError, Result};
use types::{ImageEncodedData, ImageEncodingFormat};
use util;

#[derive(Debug, PartialEq)]
pub enum EncodedImage {
    Bytes {
        format: ImageEncodingFormat,
        bytes: Rc<Vec<u8>>,
        size_info: Option<(u32, u32)>
    },
    BytesAndDataUri {
        format: ImageEncodingFormat,
        bytes: Rc<Vec<u8>>,
        data_uri: Rc<String>,
        size_info: Option<(u32, u32)>
    }
}

impl EncodedImage {
    pub fn guess_format(bytes: &[u8]) -> Result<ImageEncodingFormat> {
        Ok(image::guess_format(bytes)?.into())
    }

    pub fn get_dimensions(format: ImageEncodingFormat, bytes: &[u8]) -> Result<(u32, u32)> {
        util::get_dimensions(format, bytes)
    }
}

impl TEncodedImage for EncodedImage {
    type Error = ImageError;

    fn from_bytes<T>(bytes: T) -> Result<Self>
    where
        T: Into<Rc<Vec<u8>>>
    {
        let bytes = bytes.into();
        let format = EncodedImage::guess_format(&bytes)?;
        let size_info = None;
        Ok(EncodedImage::Bytes {
            format,
            bytes,
            size_info
        })
    }

    fn from_data_uri<T>(data_uri: T) -> Result<Self>
    where
        T: Into<Rc<String>>
    {
        let data_uri = data_uri.into();
        let bytes = Rc::new(base64_util::from_data_uri(&data_uri).map_err(|_| ImageError::DataUriDecodeError)?);
        let format = EncodedImage::guess_format(&bytes)?;
        let size_info = None;
        Ok(EncodedImage::BytesAndDataUri {
            format,
            bytes,
            data_uri,
            size_info
        })
    }

    fn format(&self) -> Option<ImageEncodingFormat> {
        match self {
            &EncodedImage::Bytes { format, .. } | &EncodedImage::BytesAndDataUri { format, .. } => Some(format)
        }
    }

    fn bytes(&self) -> Option<&Rc<Vec<u8>>> {
        match self {
            &EncodedImage::Bytes { ref bytes, .. } | &EncodedImage::BytesAndDataUri { ref bytes, .. } => Some(bytes)
        }
    }

    fn data_uri(&self) -> Option<&Rc<String>> {
        match self {
            &EncodedImage::Bytes { .. } => None,
            &EncodedImage::BytesAndDataUri { ref data_uri, .. } => Some(data_uri)
        }
    }

    fn size_info(&self) -> Option<(u32, u32)> {
        match self {
            &EncodedImage::Bytes { size_info, .. } | &EncodedImage::BytesAndDataUri { size_info, .. } => size_info
        }
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn info(&self) -> ImageEncodedData {
        match self {
            &EncodedImage::Bytes { ref bytes, format, .. } => ImageEncodedData::Bytes { bytes, format },
            &EncodedImage::BytesAndDataUri { ref data_uri, .. } => ImageEncodedData::DataUri { data_uri }
        }
    }
}

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

use std::io;
use std::result;

use image;

pub type Result<T> = result::Result<T, ImageError>;

#[derive(Debug)]
pub enum ImageError {
    IOError(io::Error),
    LibError(image::ImageError),
    DataUriDecodeError,
    ImageAlreadyAdded
}

impl From<io::Error> for ImageError {
    fn from(err: io::Error) -> Self {
        ImageError::IOError(err)
    }
}

impl From<image::ImageError> for ImageError {
    fn from(err: image::ImageError) -> Self {
        ImageError::LibError(err)
    }
}

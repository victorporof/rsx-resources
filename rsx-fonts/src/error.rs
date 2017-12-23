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
use std::str;

use freetype::freetype as ft;

pub type Result<T> = result::Result<T, FontError>;

#[derive(Debug)]
pub enum FontError {
    FTError(ft::FT_Error),
    IOError(io::Error),
    Utf8Error(str::Utf8Error),
    DataUriDecodeError,
    FaceAlreadyAdded,
    FontInstanceAlreadyAdded,
    FaceNotFound,
    FaceNotLoaded,
    FaceFamilyNameMissing,
    FaceSizeMissing,
    FaceGlyphMissing
}

impl From<ft::FT_Error> for FontError {
    fn from(err: ft::FT_Error) -> Self {
        FontError::FTError(err)
    }
}

impl From<io::Error> for FontError {
    fn from(err: io::Error) -> Self {
        FontError::IOError(err)
    }
}

impl From<str::Utf8Error> for FontError {
    fn from(err: str::Utf8Error) -> Self {
        FontError::Utf8Error(err)
    }
}

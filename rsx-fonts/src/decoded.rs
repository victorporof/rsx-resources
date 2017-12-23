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

use rsx_shared::traits::TEncodedFont;

use types::FontResourceData;

#[derive(Debug, PartialEq)]
pub struct DecodedFont {
    pub bytes: Rc<Vec<u8>>,
    pub face_index: usize
}

impl DecodedFont {
    pub fn from_raw_parts(bytes: Rc<Vec<u8>>, face_index: usize) -> DecodedFont {
        DecodedFont { bytes, face_index }
    }

    pub fn from_encoded_font<E>(encoded: &E, face_index: usize) -> DecodedFont
    where
        E: TEncodedFont
    {
        Self::from_raw_parts(Rc::clone(encoded.bytes().unwrap()), face_index)
    }

    pub fn info(&self) -> FontResourceData {
        FontResourceData {
            bytes: &self.bytes,
            face_index: self.face_index
        }
    }
}

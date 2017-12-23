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

use std::ffi::CString;
use std::mem;
use std::os::raw::c_char;
use std::rc::Rc;

use base64_util;
use rsx_shared::traits::{TFontInstanceKey, TFontKey, TFontKeysAPI, TGlyphInstance, TImageKeysAPI, TMediaKey};
use rsx_shared::types::{FontEncodedData, FontInstanceResourceData, FontResourceData, ImageEncodedData, ImageResourceData};
use serde::Serialize;
use serde_json;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DefaultImageKey(pub u64);

impl TMediaKey for DefaultImageKey {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DefaultFontKey(pub u64);

impl TFontKey for DefaultFontKey {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DefaultFontInstanceKey(pub u64);

impl TFontInstanceKey for DefaultFontInstanceKey {}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct DefaultGlyphInstance {
    pub glyph_index: u32,
    pub x_64: i32,
    pub y_64: i32
}

impl TGlyphInstance for DefaultGlyphInstance {
    fn new(glyph_index: u32, x_64: i32, y_64: i32) -> Self {
        DefaultGlyphInstance {
            glyph_index,
            x_64,
            y_64
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DefaultImageKeysAPI {
    up: <DefaultImageKeysAPI as TImageKeysAPI>::ResourceUpdates,
    next_image_key: u64
}

impl TImageKeysAPI for DefaultImageKeysAPI {
    type RootRendererAPI = ();
    type ResourceUpdates = ResourceUpdates<Self::ImageKey, DefaultFontKey, DefaultFontInstanceKey>;
    type ImageKey = DefaultImageKey;

    fn new(_: Self::RootRendererAPI) -> Self {
        DefaultImageKeysAPI {
            up: Self::ResourceUpdates::with_capacity(0),
            next_image_key: 0
        }
    }

    fn add_image(&mut self, encoded: ImageEncodedData, _: ImageResourceData) -> Self::ImageKey {
        let image_key = DefaultImageKey(self.next_image_key);
        self.next_image_key += 1;

        let uri = match encoded {
            ImageEncodedData::Bytes { format, bytes } => Rc::new(base64_util::to_image_data_uri(format.as_ref(), bytes)),
            ImageEncodedData::DataUri { data_uri } => Rc::clone(data_uri)
        };

        self.up.add_image(image_key, uri);

        image_key
    }

    fn take_resource_updates(&mut self) -> Self::ResourceUpdates {
        mem::replace(&mut self.up, Self::ResourceUpdates::with_capacity(0))
    }
}

#[derive(Debug, PartialEq)]
pub struct DefaultFontKeysAPI {
    up: <DefaultFontKeysAPI as TFontKeysAPI>::ResourceUpdates,
    next_font_key: u64,
    next_font_instance_key: u64
}

impl TFontKeysAPI for DefaultFontKeysAPI {
    type RootRendererAPI = ();
    type ResourceUpdates = ResourceUpdates<DefaultImageKey, Self::FontKey, Self::FontInstanceKey>;
    type FontKey = DefaultFontKey;
    type FontInstanceKey = DefaultFontInstanceKey;
    type GlyphInstance = DefaultGlyphInstance;

    fn new(_: Self::RootRendererAPI) -> Self {
        DefaultFontKeysAPI {
            up: Self::ResourceUpdates::with_capacity(0),
            next_font_key: 0,
            next_font_instance_key: 0
        }
    }

    fn add_font(&mut self, encoded: FontEncodedData, _: FontResourceData) -> Self::FontKey {
        let font_key = DefaultFontKey(self.next_font_key);
        self.next_font_key += 1;

        let uri = match encoded {
            FontEncodedData::Bytes { bytes } => Rc::new(base64_util::to_font_data_uri(bytes)),
            FontEncodedData::DataUri { data_uri } => Rc::clone(data_uri)
        };

        self.up.add_font(font_key, uri);

        font_key
    }

    fn add_font_instance(&mut self, font_key: Self::FontKey, resource: FontInstanceResourceData) -> Self::FontInstanceKey {
        let font_instance_key = DefaultFontInstanceKey(self.next_font_instance_key);
        self.next_font_instance_key += 1;

        let size = resource.size;
        self.up.add_font_instance(font_instance_key, font_key, size);

        font_instance_key
    }

    fn take_resource_updates(&mut self) -> Self::ResourceUpdates {
        mem::replace(&mut self.up, Self::ResourceUpdates::with_capacity(0))
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ResourceUpdates<ImageKey, FontKey, FontInstanceKey> {
    pub updates: Vec<Update<ImageKey, FontKey, FontInstanceKey>>
}

impl<ImageKey, FontKey, FontInstanceKey> ResourceUpdates<ImageKey, FontKey, FontInstanceKey> {
    pub fn with_capacity(capacity: usize) -> Self {
        ResourceUpdates {
            updates: Vec::with_capacity(capacity)
        }
    }

    pub fn add_image(&mut self, key: ImageKey, data_uri: Rc<String>) {
        self.updates.push(Update::AddImage { key, data_uri });
    }

    pub fn add_font(&mut self, key: FontKey, data_uri: Rc<String>) {
        self.updates.push(Update::AddFont { key, data_uri });
    }

    pub fn add_font_instance(&mut self, instance_key: FontInstanceKey, key: FontKey, size: u32) {
        self.updates.push(Update::AddFontInstance {
            instance_key,
            key,
            size
        });
    }

    pub fn len(&self) -> usize {
        self.updates.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn merge(&mut self, mut other: Self) {
        self.updates.append(&mut other.updates);
    }
}

impl<ImageKey, FontKey, FontInstanceKey> Into<String> for ResourceUpdates<ImageKey, FontKey, FontInstanceKey>
where
    ImageKey: Serialize,
    FontKey: Serialize,
    FontInstanceKey: Serialize
{
    #[cfg(not(feature = "pretty-json-mode"))]
    fn into(self) -> String {
        serde_json::to_string(&self.updates).unwrap_or_else(|_| "".to_string())
    }

    #[cfg(feature = "pretty-json-mode")]
    fn into(self) -> String {
        serde_json::to_string_pretty(&self.updates).unwrap_or_else(|_| "".to_string())
    }
}

impl<ImageKey, FontKey, FontInstanceKey> Into<*mut c_char> for ResourceUpdates<ImageKey, FontKey, FontInstanceKey>
where
    ImageKey: Serialize,
    FontKey: Serialize,
    FontInstanceKey: Serialize
{
    fn into(self) -> *mut c_char {
        let string: String = self.into();
        unsafe { CString::from_vec_unchecked(string.into_bytes()) }.into_raw()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Update<ImageKey, FontKey, FontInstanceKey> {
    AddImage {
        key: ImageKey,
        data_uri: Rc<String>
    },
    AddFont {
        key: FontKey,
        data_uri: Rc<String>
    },
    AddFontInstance {
        key: FontKey,
        instance_key: FontInstanceKey,
        size: u32
    }
}

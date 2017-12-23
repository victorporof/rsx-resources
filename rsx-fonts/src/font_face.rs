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

use std::ffi::CStr;
use std::os::raw::c_uint;
use std::ptr;
use std::rc::Rc;

use freetype::freetype::{
    self,
    FT_F26Dot6,
    FT_Face,
    FT_Get_Char_Index,
    FT_Glyph_Metrics,
    FT_Int32,
    FT_Library,
    FT_Load_Glyph,
    FT_Long,
    FT_New_Memory_Face,
    FT_Set_Char_Size,
    FT_Size_Metrics,
    FT_UInt,
    FT_ULong
};

use error::{FontError, Result};

bitflags! {
    pub struct LoadFlag: c_uint {
        const DEFAULT = freetype::FT_LOAD_DEFAULT;
        const NO_SCALE = freetype::FT_LOAD_NO_SCALE;
        const NO_HINTING = freetype::FT_LOAD_NO_HINTING;
        const RENDER = freetype::FT_LOAD_RENDER;
        const NO_BITMAP = freetype::FT_LOAD_NO_BITMAP;
        const VERTICAL_LAYOUT = freetype::FT_LOAD_VERTICAL_LAYOUT;
        const FORCE_AUTOHINT = freetype::FT_LOAD_FORCE_AUTOHINT;
        const CROP_BITMAP = freetype::FT_LOAD_CROP_BITMAP;
        const PEDANTIC = freetype::FT_LOAD_PEDANTIC;
        const IGNORE_GLOBAL_ADVANCE_WIDTH = freetype::FT_LOAD_IGNORE_GLOBAL_ADVANCE_WIDTH;
        const NO_RECURSE = freetype::FT_LOAD_NO_RECURSE;
        const IGNORE_TRANSFORM = freetype::FT_LOAD_IGNORE_TRANSFORM;
        const MONOCHROME = freetype::FT_LOAD_MONOCHROME;
        const LINEAR_DESIGN = freetype::FT_LOAD_LINEAR_DESIGN;
        const NO_AUTOHINT = freetype::FT_LOAD_NO_AUTOHINT;
        const COLOR = freetype::FT_LOAD_COLOR;
        const COMPUTE_METRICS = freetype::FT_LOAD_COMPUTE_METRICS;
        const SBITS_ONLY = freetype::FT_LOAD_SBITS_ONLY;
    }
}

#[derive(Debug, PartialEq)]
pub struct FontFace {
    raw: FT_Face,
    bytes: Rc<Vec<u8>>,
    face_index: usize
}

impl FontFace {
    pub fn new(lib: &FT_Library, bytes: &Rc<Vec<u8>>, face_index: usize) -> Result<Self> {
        let mut raw: FT_Face = ptr::null_mut();
        let result = unsafe {
            FT_New_Memory_Face(
                *lib,
                bytes.as_ptr(),
                bytes.len() as FT_Long,
                face_index as FT_Long,
                &mut raw
            )
        };
        if !result.succeeded() || raw.is_null() {
            Err(result)?
        } else {
            Ok(FontFace {
                raw,
                bytes: Rc::clone(bytes),
                face_index
            })
        }
    }

    pub fn get_bytes(&self) -> Rc<Vec<u8>> {
        Rc::clone(&self.bytes)
    }

    pub fn get_face_index(&self) -> usize {
        self.face_index
    }

    pub fn get_family_name(&self) -> Result<&str> {
        let face = unsafe { self.raw.as_ref() }.ok_or(FontError::FaceNotLoaded)?;
        let family_name_ptr = unsafe { face.family_name.as_ref() }.ok_or(FontError::FaceFamilyNameMissing)?;
        let family_name_str = unsafe { CStr::from_ptr(family_name_ptr).to_str() }?;
        Ok(family_name_str)
    }

    pub fn get_char_index(&self, c: char) -> u32 {
        unsafe { FT_Get_Char_Index(self.raw, c as FT_ULong) }
    }

    pub fn set_char_size(&self, w: usize, h: usize, h_res: u32, v_res: u32) -> Result<()> {
        let result = unsafe { FT_Set_Char_Size(self.raw, w as FT_F26Dot6, h as FT_F26Dot6, h_res, v_res) };
        if !result.succeeded() {
            Err(result)?
        } else {
            Ok(())
        }
    }

    pub fn load_glyph(&self, c: u32, flags: LoadFlag) -> Result<()> {
        let result = unsafe { FT_Load_Glyph(self.raw, c as FT_UInt, flags.bits as FT_Int32) };
        if !result.succeeded() {
            Err(result)?
        } else {
            Ok(())
        }
    }

    pub fn get_size_metrics(&self) -> Result<FT_Size_Metrics> {
        let face = unsafe { self.raw.as_ref() }.ok_or(FontError::FaceNotLoaded)?;
        let size = unsafe { face.size.as_ref() }.ok_or(FontError::FaceSizeMissing)?;
        Ok(size.metrics)
    }

    pub fn get_glyph_metrics(&self) -> Result<FT_Glyph_Metrics> {
        let face = unsafe { self.raw.as_ref() }.ok_or(FontError::FaceNotLoaded)?;
        let glyph_slot = unsafe { face.glyph.as_ref() }.ok_or(FontError::FaceGlyphMissing)?;
        Ok(glyph_slot.metrics)
    }
}

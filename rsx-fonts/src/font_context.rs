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

use std::collections::hash_map::Entry;
use std::hash::{Hash, Hasher};
use std::ptr;
use std::rc::Rc;

use fnv::{FnvHashMap, FnvHasher};
use freetype::freetype::{FT_Init_FreeType, FT_Library};
use rsx_shared::traits::{TFontInstanceKey, TFontKey, TGlyphInstance};

use error::{FontError, Result};
use font_face::{FontFace, LoadFlag};
use types::{FontId, FontInstance, FontSizeMetrics, GlyphDimensions, GlyphStore, GlyphsArray};

#[derive(Debug, PartialEq)]
pub struct FontContext {
    library: FT_Library,
    faces: FnvHashMap<FontId, FontFace>
}

impl FontContext {
    pub fn new() -> Result<Self> {
        let mut library: FT_Library = ptr::null_mut();
        let result = unsafe { FT_Init_FreeType(&mut library) };
        if !result.succeeded() {
            Err(result)?
        } else {
            Ok(FontContext {
                library,
                faces: FnvHashMap::default()
            })
        }
    }

    pub fn add_face(&mut self, font_id: FontId, bytes: &Rc<Vec<u8>>, face_index: usize) -> Result<()> {
        match self.faces.entry(font_id) {
            Entry::Occupied(_) => {
                Err(FontError::FaceAlreadyAdded)?;
            }
            Entry::Vacant(e) => {
                e.insert(FontFace::new(&self.library, bytes, face_index)?);
            }
        }

        Ok(())
    }

    pub fn get_bytes(&self, font_id: FontId) -> Result<Rc<Vec<u8>>> {
        self.faces
            .get(&font_id)
            .ok_or(FontError::FaceNotFound)
            .map(|f| f.get_bytes())
    }

    pub fn get_face_index(&self, font_id: FontId) -> Result<usize> {
        self.faces
            .get(&font_id)
            .ok_or(FontError::FaceNotFound)
            .map(|f| f.get_face_index())
    }

    pub fn get_family_name(&self, font_id: FontId) -> Result<&str> {
        self.faces
            .get(&font_id)
            .ok_or(FontError::FaceNotFound)
            .and_then(|f| f.get_family_name())
    }

    pub fn get_glyph_index<FontKey, FontInstanceKey, GlyphInstance>(
        &self,
        instance: &FontInstance<FontKey, FontInstanceKey, GlyphInstance>,
        c: char
    ) -> Result<u32> {
        let font_id = instance.font_id();
        let face = self.faces.get(&font_id).ok_or(FontError::FaceNotFound)?;

        Ok(face.get_char_index(c))
    }

    pub fn get_glyph_dimensions<FontKey, FontInstanceKey, GlyphInstance>(
        &self,
        instance: &FontInstance<FontKey, FontInstanceKey, GlyphInstance>,
        c: char
    ) -> Result<GlyphDimensions> {
        let font_id = instance.font_id();
        let face = self.faces.get(&font_id).ok_or(FontError::FaceNotFound)?;
        let point_size = (instance.size() * 64) as usize;
        let glyph_index = self.get_glyph_index(instance, c)?;

        face.set_char_size(point_size, 0, instance.dpi(), 0)?;
        face.load_glyph(glyph_index, LoadFlag::NO_HINTING | LoadFlag::NO_BITMAP)?;
        let metrics = face.get_glyph_metrics()?;

        Ok(GlyphDimensions {
            glyph_index,
            width_64: metrics.width as i32,
            height_64: metrics.height as i32,
            hori_advance_64: metrics.horiAdvance as i32,
            vert_advance_64: metrics.vertAdvance as i32
        })
    }

    pub fn get_global_size_metrics<FontKey, FontInstanceKey, GlyphInstance>(
        &self,
        instance: &FontInstance<FontKey, FontInstanceKey, GlyphInstance>
    ) -> Result<FontSizeMetrics> {
        let font_id = instance.font_id();

        let face = self.faces.get(&font_id).ok_or(FontError::FaceNotFound)?;
        let point_size = (instance.size() * 64) as usize;

        face.set_char_size(point_size, 0, instance.dpi(), 0)?;
        let face_metrics = face.get_size_metrics()?;

        Ok(FontSizeMetrics {
            nominal_width: face_metrics.x_ppem,
            nominal_height: face_metrics.y_ppem,
            ascender_64: face_metrics.ascender as i32,
            descender_64: face_metrics.descender as i32,
            height_64: face_metrics.height as i32,
            max_advance_64: face_metrics.max_advance as i32
        })
    }

    pub fn shape_text_h<T, FontKey, FontInstanceKey, GlyphInstance>(
        &self,
        instance: &FontInstance<FontKey, FontInstanceKey, GlyphInstance>,
        text: T
    ) -> Result<GlyphStore<FontKey, FontInstanceKey, GlyphInstance>>
    where
        T: AsRef<str>,
        FontKey: TFontKey,
        FontInstanceKey: TFontInstanceKey,
        GlyphInstance: TGlyphInstance
    {
        let text = text.as_ref();

        let mut hasher = FnvHasher::default();
        // instance.external_key().hash(&mut hasher);
        // instance.external_instance_key().hash(&mut hasher);
        text.hash(&mut hasher);

        let generation_id = hasher.finish();
        let mut cache = instance.shaped_text_h_cache.borrow_mut();

        match cache.entry(generation_id) {
            Entry::Occupied(e) => Ok(GlyphStore::clone(e.get())),
            Entry::Vacant(e) => {
                let mut glyphs = Vec::with_capacity(text.len());
                let mut pen_position_64 = 0;
                let font_size_metrics = self.get_global_size_metrics(instance)?;
                let pen_baseline_64 = font_size_metrics.ascender_64;

                for c in text.chars() {
                    let GlyphDimensions {
                        glyph_index,
                        hori_advance_64,
                        ..
                    } = self.get_glyph_dimensions(instance, c)?;

                    glyphs.push(GlyphInstance::new(
                        glyph_index,
                        pen_position_64,
                        pen_baseline_64
                    ));
                    pen_position_64 += hori_advance_64;
                }

                Ok(GlyphStore::clone(
                    e.insert(GlyphStore {
                        generation_id,
                        font_key: instance.external_key(),
                        font_instance_key: instance.external_instance_key(),
                        width_64: pen_position_64,
                        height_64: font_size_metrics.height_64,
                        glyphs: GlyphsArray(Rc::from(glyphs.into_boxed_slice()))
                    })
                ))
            }
        }
    }

    pub fn shape_text_v<T, FontKey, FontInstanceKey, GlyphInstance>(
        &self,
        instance: &FontInstance<FontKey, FontInstanceKey, GlyphInstance>,
        text: T
    ) -> Result<GlyphStore<FontKey, FontInstanceKey, GlyphInstance>>
    where
        T: AsRef<str>,
        FontKey: TFontKey,
        FontInstanceKey: TFontInstanceKey,
        GlyphInstance: TGlyphInstance
    {
        let text = text.as_ref();

        let mut hasher = FnvHasher::default();
        // instance.external_key().hash(&mut hasher);
        // instance.external_instance_key().hash(&mut hasher);
        text.hash(&mut hasher);

        let generation_id = hasher.finish();
        let mut cache = instance.shaped_text_v_cache.borrow_mut();

        match cache.entry(generation_id) {
            Entry::Occupied(e) => Ok(GlyphStore::clone(e.get())),
            Entry::Vacant(e) => {
                let mut glyphs = Vec::with_capacity(text.len());
                let mut pen_position_64 = 0;
                let font_size_metrics = self.get_global_size_metrics(instance)?;

                for c in text.chars() {
                    let GlyphDimensions {
                        glyph_index,
                        vert_advance_64,
                        ..
                    } = self.get_glyph_dimensions(instance, c)?;

                    glyphs.push(GlyphInstance::new(glyph_index, 0, pen_position_64));
                    pen_position_64 += vert_advance_64;
                }

                Ok(GlyphStore::clone(
                    e.insert(GlyphStore {
                        generation_id: hasher.finish(),
                        font_key: instance.external_key(),
                        font_instance_key: instance.external_instance_key(),
                        width_64: font_size_metrics.max_advance_64,
                        height_64: pen_position_64,
                        glyphs: GlyphsArray(Rc::from(glyphs.into_boxed_slice()))
                    })
                ))
            }
        }
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    extern crate rsx_resource_updates;

    use super::*;
    use rsx_resource_updates::types::{
        DefaultFontInstanceKey as FontInstanceKey,
        DefaultFontKey as FontKey,
        DefaultGlyphInstance as GlyphInstance
    };
    use rsx_shared::traits::TGlyphStore;

    #[test]
    fn test_fonts_context_1() {
        let mut font_context = FontContext::new().unwrap();

        let font_id = FontId::new("FreeSans");
        let font_bytes = Rc::new(include_bytes!("../../rsx-resource-group/tests/fixtures/FreeSans.ttf").to_vec());
        assert!(font_context.add_face(font_id, &font_bytes, 0).is_ok());
        assert!(font_context.add_face(font_id, &font_bytes, 0).is_err());

        assert_eq!(font_context.get_bytes(font_id).unwrap(), font_bytes);
        assert_eq!(font_context.get_face_index(font_id).unwrap(), 0);
    }

    #[test]
    fn test_fonts_context_2() {
        let mut font_context = FontContext::new().unwrap();

        let font_bytes_1 = Rc::new(include_bytes!("../../rsx-resource-group/tests/fixtures/FreeSans.ttf").to_vec());
        let font_id_1 = FontId::new("FreeSans");
        assert!(font_context.add_face(font_id_1, &font_bytes_1, 0).is_ok());
        assert!(font_context.add_face(font_id_1, &font_bytes_1, 0).is_err());

        let font_bytes_2 = Rc::new(include_bytes!("../../rsx-resource-group/tests/fixtures/FiraMono-Regular.ttf").to_vec());
        let font_id_2 = FontId::new("FiraMono");
        assert!(font_context.add_face(font_id_2, &font_bytes_2, 0).is_ok());
        assert!(font_context.add_face(font_id_2, &font_bytes_2, 0).is_err());

        assert_eq!(font_context.get_bytes(font_id_1).unwrap(), font_bytes_1);
        assert_eq!(font_context.get_face_index(font_id_1).unwrap(), 0);

        assert_eq!(font_context.get_bytes(font_id_2).unwrap(), font_bytes_2);
        assert_eq!(font_context.get_face_index(font_id_2).unwrap(), 0);
    }

    #[test]
    fn test_fonts_raw_simple() {
        let mut font_context = FontContext::new().unwrap();

        let font_id = FontId::new("FreeSans");
        let font_bytes = Rc::new(include_bytes!("../../rsx-resource-group/tests/fixtures/FreeSans.ttf").to_vec());
        assert!(font_context.add_face(font_id, &font_bytes, 0).is_ok());

        let face = font_context.faces.get(&font_id).unwrap();
        assert_eq!(face.get_family_name().unwrap(), "FreeSans");
        assert_eq!(face.get_char_index('a'), 68);
    }

    #[test]
    fn test_fonts_simple_1() {
        let mut font_context = FontContext::new().unwrap();

        let font_id = FontId::new("FreeSans");
        let font_bytes = Rc::new(include_bytes!("../../rsx-resource-group/tests/fixtures/FreeSans.ttf").to_vec());
        assert!(font_context.add_face(font_id, &font_bytes, 0).is_ok());

        let instance = FontInstance::<_, _, ()>::new(font_id, 12, 72, (), ());
        assert_eq!(font_context.get_glyph_index(&instance, 'a').unwrap(), 68);

        assert_eq!(
            font_context.get_glyph_dimensions(&instance, 'a').unwrap(),
            GlyphDimensions {
                glyph_index: 68,
                width_64: 379,
                height_64: 432,
                hori_advance_64: 427,
                vert_advance_64: 768
            }
        );

        assert_eq!(
            font_context.get_global_size_metrics(&instance).unwrap(),
            FontSizeMetrics {
                nominal_width: 12,
                nominal_height: 12,
                ascender_64: 768,
                descender_64: -256,
                height_64: 1088,
                max_advance_64: 1152
            }
        );
    }

    #[test]
    fn test_fonts_simple_2() {
        let mut font_context = FontContext::new().unwrap();

        let font_id = FontId::new("FreeSans");
        let font_bytes = Rc::new(include_bytes!("../../rsx-resource-group/tests/fixtures/FreeSans.ttf").to_vec());
        assert!(font_context.add_face(font_id, &font_bytes, 0).is_ok());

        let instance = FontInstance::<_, _, ()>::new(font_id, 16, 72, (), ());
        assert_eq!(font_context.get_glyph_index(&instance, 'a').unwrap(), 68);

        assert_eq!(
            font_context.get_glyph_dimensions(&instance, 'a').unwrap(),
            GlyphDimensions {
                glyph_index: 68,
                width_64: 505,
                height_64: 576,
                hori_advance_64: 569,
                vert_advance_64: 1024
            }
        );

        assert_eq!(
            font_context.get_global_size_metrics(&instance).unwrap(),
            FontSizeMetrics {
                nominal_width: 16,
                nominal_height: 16,
                ascender_64: 1024,
                descender_64: -320,
                height_64: 1408,
                max_advance_64: 1536
            }
        );
    }

    #[test]
    fn test_fonts_simple_3a() {
        let mut font_context = FontContext::new().unwrap();

        let font_id = FontId::new("FreeSans");
        let font_bytes = Rc::new(include_bytes!("../../rsx-resource-group/tests/fixtures/FreeSans.ttf").to_vec());
        assert!(font_context.add_face(font_id, &font_bytes, 0).is_ok());

        let instance = FontInstance::new(font_id, 16, 72, FontKey(0), FontInstanceKey(0));
        assert_eq!(font_context.get_glyph_index(&instance, 'a').unwrap(), 68);

        let shaped_text = font_context.shape_text_h(&instance, "Hello world").unwrap();
        assert_eq!(shaped_text.width_f(), 79.078125);
        assert_eq!(shaped_text.height_f(), 22.0);
        assert_eq!(shaped_text.font_key, instance.external_key());
        assert_eq!(
            shaped_text.font_instance_key,
            instance.external_instance_key()
        );
        assert_eq!(
            shaped_text.glyphs.0,
            Rc::from(
                vec![
                    GlyphInstance {
                        glyph_index: 43,
                        x_64: 0,
                        y_64: 1024
                    },
                    GlyphInstance {
                        glyph_index: 72,
                        x_64: 739,
                        y_64: 1024
                    },
                    GlyphInstance {
                        glyph_index: 79,
                        x_64: 1308,
                        y_64: 1024
                    },
                    GlyphInstance {
                        glyph_index: 79,
                        x_64: 1535,
                        y_64: 1024
                    },
                    GlyphInstance {
                        glyph_index: 82,
                        x_64: 1762,
                        y_64: 1024
                    },
                    GlyphInstance {
                        glyph_index: 3,
                        x_64: 2331,
                        y_64: 1024
                    },
                    GlyphInstance {
                        glyph_index: 90,
                        x_64: 2616,
                        y_64: 1024
                    },
                    GlyphInstance {
                        glyph_index: 82,
                        x_64: 3355,
                        y_64: 1024
                    },
                    GlyphInstance {
                        glyph_index: 85,
                        x_64: 3924,
                        y_64: 1024
                    },
                    GlyphInstance {
                        glyph_index: 79,
                        x_64: 4265,
                        y_64: 1024
                    },
                    GlyphInstance {
                        glyph_index: 71,
                        x_64: 4492,
                        y_64: 1024
                    },
                ].into_boxed_slice()
            )
        );
    }

    #[test]
    fn test_fonts_simple_3b() {
        let mut font_context = FontContext::new().unwrap();

        let font_id = FontId::new("FreeSans");
        let font_bytes = Rc::new(include_bytes!("../../rsx-resource-group/tests/fixtures/FreeSans.ttf").to_vec());
        assert!(font_context.add_face(font_id, &font_bytes, 0).is_ok());

        let instance = FontInstance::new(font_id, 16, 72, FontKey(0), FontInstanceKey(0));
        assert_eq!(font_context.get_glyph_index(&instance, 'a').unwrap(), 68);
        let shaped_text = font_context.shape_text_v(&instance, "Hello world").unwrap();
        assert_eq!(shaped_text.width_f(), 24.0);
        assert_eq!(shaped_text.height_f(), 176.0);
        assert_eq!(shaped_text.font_key, instance.external_key());
        assert_eq!(
            shaped_text.font_instance_key,
            instance.external_instance_key()
        );
        assert_eq!(
            shaped_text.glyphs.0,
            Rc::from(
                vec![
                    GlyphInstance {
                        glyph_index: 43,
                        x_64: 0,
                        y_64: 0
                    },
                    GlyphInstance {
                        glyph_index: 72,
                        x_64: 0,
                        y_64: 1024
                    },
                    GlyphInstance {
                        glyph_index: 79,
                        x_64: 0,
                        y_64: 2048
                    },
                    GlyphInstance {
                        glyph_index: 79,
                        x_64: 0,
                        y_64: 3072
                    },
                    GlyphInstance {
                        glyph_index: 82,
                        x_64: 0,
                        y_64: 4096
                    },
                    GlyphInstance {
                        glyph_index: 3,
                        x_64: 0,
                        y_64: 5120
                    },
                    GlyphInstance {
                        glyph_index: 90,
                        x_64: 0,
                        y_64: 6144
                    },
                    GlyphInstance {
                        glyph_index: 82,
                        x_64: 0,
                        y_64: 7168
                    },
                    GlyphInstance {
                        glyph_index: 85,
                        x_64: 0,
                        y_64: 8192
                    },
                    GlyphInstance {
                        glyph_index: 79,
                        x_64: 0,
                        y_64: 9216
                    },
                    GlyphInstance {
                        glyph_index: 71,
                        x_64: 0,
                        y_64: 10240
                    },
                ].into_boxed_slice()
            )
        );
    }
}

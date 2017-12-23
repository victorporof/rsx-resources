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

use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Deref;
use std::rc::Rc;

use fnv::{FnvHashMap, FnvHasher};
use rsx_shared::consts::{DEFAULT_FONT_DPI, DEFAULT_FONT_SIZE};
use rsx_shared::traits::{TEncodedFont, TFontCache, TFontInstanceKey, TFontKey, TFontKeysAPI, TGlyphInstance, TGlyphStore};
use uuid::Uuid;

use error::{FontError, Result};
use font_context::FontContext;

pub use decoded::DecodedFont;
pub use encoded::EncodedFont;
pub use rsx_shared::types::{FontEncodedData, FontInstanceResourceData, FontResourceData};

pub type TFontInstance<A> =
    FontInstance<<A as TFontKeysAPI>::FontKey, <A as TFontKeysAPI>::FontInstanceKey, <A as TFontKeysAPI>::GlyphInstance>;

pub type RcFontInstance<A> = Rc<TFontInstance<A>>;
pub type FontInstanceRef<'a, A> = &'a TFontInstance<A>;
pub type FontInstanceRefMut<'a, A> = &'a mut TFontInstance<A>;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FontId(u64);

impl FontId {
    pub fn new<P>(font_name: P) -> Self
    where
        P: AsRef<str>
    {
        let mut hasher = FnvHasher::default();
        hasher.write(font_name.as_ref().as_bytes());
        FontId(hasher.finish())
    }

    pub fn uuid() -> Self {
        let mut hasher = FnvHasher::default();
        Uuid::new_v4().hash(&mut hasher);
        FontId(hasher.finish())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FontInstanceId {
    family_name: u64,
    size: u32,
    dpi: u32
}

impl FontInstanceId {
    pub fn from_family_hash(family_name: u64, size: u32, dpi: u32) -> Self {
        FontInstanceId {
            family_name,
            size,
            dpi
        }
    }

    pub fn from_family_str<T>(family_name: T, size: u32, dpi: u32) -> Self
    where
        T: AsRef<str>
    {
        let mut hasher = FnvHasher::default();
        hasher.write(family_name.as_ref().as_bytes());
        Self::from_family_hash(hasher.finish(), size, dpi)
    }

    pub fn resize(&self, size: u32) -> Self {
        Self::from_family_hash(self.family_name, size, self.dpi)
    }

    pub fn resize_dpi(&self, size: u32, dpi: u32) -> Self {
        Self::from_family_hash(self.family_name, size, dpi)
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct FontSizeMetrics {
    pub nominal_width: u16,
    pub nominal_height: u16,
    pub ascender_64: i32,
    pub descender_64: i32,
    pub height_64: i32,
    pub max_advance_64: i32
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct GlyphDimensions {
    pub glyph_index: u32,
    pub width_64: i32,
    pub height_64: i32,
    pub hori_advance_64: i32,
    pub vert_advance_64: i32
}

#[derive(Debug, PartialEq, Clone)]
pub struct GlyphsArray<GlyphInstance>(pub(crate) Rc<[GlyphInstance]>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlyphStore<FontKey, FontInstanceKey, GlyphInstance> {
    pub(crate) font_key: FontKey,
    pub(crate) font_instance_key: FontInstanceKey,
    pub(crate) width_64: i32,
    pub(crate) height_64: i32,
    pub(crate) glyphs: GlyphsArray<GlyphInstance>,
    pub(crate) generation_id: u64
}

// Testing equality between glyph stores can be slow in the worst case scenario,
// depending on the length of the text. Use the generation id for a faster path.
impl<FontKey, FontInstanceKey, GlyphInstance> PartialEq for GlyphStore<FontKey, FontInstanceKey, GlyphInstance> {
    fn eq(&self, other: &Self) -> bool {
        self.generation_id == other.generation_id
    }
}

impl<FontKey, FontInstanceKey, GlyphInstance> TGlyphStore for GlyphStore<FontKey, FontInstanceKey, GlyphInstance>
where
    FontKey: TFontKey,
    FontInstanceKey: TFontInstanceKey,
    GlyphInstance: TGlyphInstance
{
    type FontKey = FontKey;
    type FontInstanceKey = FontInstanceKey;
    type Glyph = GlyphInstance;

    fn font_key(&self) -> Self::FontKey {
        self.font_key
    }

    fn font_instance_key(&self) -> Self::FontInstanceKey {
        self.font_instance_key
    }

    fn width_f(&self) -> f32 {
        self.width_64 as f32 / 64.0
    }

    fn height_f(&self) -> f32 {
        self.height_64 as f32 / 64.0
    }

    fn width_64(&self) -> i32 {
        self.width_64
    }

    fn height_64(&self) -> i32 {
        self.height_64
    }

    fn glyphs(&self) -> &[Self::Glyph] {
        &self.glyphs.0
    }
}

#[derive(Debug)]
pub struct FontInstance<FontKey, FontInstanceKey, GlyphInstance> {
    font_id: FontId,
    size: u32,
    dpi: u32,
    external_key: FontKey,
    external_instance_key: FontInstanceKey,
    pub(crate) shaped_text_h_cache: RefCell<FnvHashMap<u64, GlyphStore<FontKey, FontInstanceKey, GlyphInstance>>>,
    pub(crate) shaped_text_v_cache: RefCell<FnvHashMap<u64, GlyphStore<FontKey, FontInstanceKey, GlyphInstance>>>
}

// Testing equality between font instances can be slow in the worst case scenario,
// depending on the state of the cache. Use the font id for a faster path.
impl<FontKey, FontInstanceKey, GlyphInstance> PartialEq for FontInstance<FontKey, FontInstanceKey, GlyphInstance> {
    fn eq(&self, other: &Self) -> bool {
        self.font_id == other.font_id && self.size == other.size && self.dpi == other.dpi
    }
}

impl<FontKey, FontInstanceKey, GlyphInstance> FontInstance<FontKey, FontInstanceKey, GlyphInstance> {
    pub fn new(font_id: FontId, size: u32, dpi: u32, external_key: FontKey, external_instance_key: FontInstanceKey) -> Rc<Self> {
        Rc::new(FontInstance {
            font_id,
            size,
            dpi,
            external_key,
            external_instance_key,
            shaped_text_h_cache: RefCell::default(),
            shaped_text_v_cache: RefCell::default()
        })
    }

    pub fn font_id(&self) -> FontId {
        self.font_id
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn dpi(&self) -> u32 {
        self.dpi
    }

    pub fn external_key(&self) -> FontKey
    where
        FontKey: Copy
    {
        self.external_key
    }

    pub fn external_instance_key(&self) -> FontInstanceKey
    where
        FontInstanceKey: Copy
    {
        self.external_instance_key
    }
}

#[derive(Debug, PartialEq)]
pub struct SharedFonts<A: TFontKeysAPI>(Rc<RefCell<FontCache<A>>>);

impl<A> From<FontCache<A>> for SharedFonts<A>
where
    A: TFontKeysAPI
{
    fn from(value: FontCache<A>) -> Self {
        SharedFonts(Rc::new(RefCell::new(value)))
    }
}

impl<A> Clone for SharedFonts<A>
where
    A: TFontKeysAPI
{
    fn clone(&self) -> Self {
        SharedFonts(Rc::clone(&self.0))
    }
}

impl<A> Deref for SharedFonts<A>
where
    A: TFontKeysAPI
{
    type Target = RefCell<FontCache<A>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<A> TFontCache for SharedFonts<A>
where
    A: TFontKeysAPI + 'static
{
    type FontInstance = RcFontInstance<A>;
    type FontId = FontId;
    type ResourceUpdates = A::ResourceUpdates;
    type Glyphs = GlyphStore<A::FontKey, A::FontInstanceKey, A::GlyphInstance>;

    fn add_raw<P, T>(&mut self, font_name: P, bytes: T, face_index: usize) -> Option<()>
    where
        P: AsRef<str>,
        T: Into<Rc<Vec<u8>>>
    {
        let id = FontId::new(font_name);
        self.borrow_mut().add_raw(id, bytes, face_index).ok()
    }

    fn add_font<P, E>(&mut self, font_name: P, encoded: &E, face_index: usize) -> Option<()>
    where
        P: AsRef<str>,
        E: TEncodedFont
    {
        let id = FontId::new(font_name);
        self.borrow_mut().add_font(id, encoded, face_index).ok()
    }

    fn add_font_with_id<E>(&mut self, id: Self::FontId, encoded: &E, face_index: usize) -> Option<()>
    where
        E: TEncodedFont
    {
        self.borrow_mut().add_font(id, encoded, face_index).ok()
    }

    fn get_family_name<P>(&self, font_name: P) -> Option<String>
    where
        P: AsRef<str>
    {
        let id = FontId::new(font_name);
        self.borrow().get_family_name_for_id(id).ok()
    }

    fn get_family_name_for_id(&self, id: Self::FontId) -> Option<String> {
        self.borrow().get_family_name_for_id(id).ok()
    }

    fn set_default_font<T>(&mut self, family_name: T, size: u32, dpi: u32)
    where
        T: AsRef<str>
    {
        self.borrow_mut().set_default_font(family_name, size, dpi);
    }

    fn get_default_font(&self) -> Option<Self::FontInstance> {
        self.borrow().get_default_font()
    }

    fn get_default_font_with_size(&self, s: u32) -> Option<Self::FontInstance> {
        self.borrow_mut().get_default_font_with_size(s)
    }

    fn get_default_font_with_size_and_dpi(&self, s: u32, d: u32) -> Option<Self::FontInstance> {
        self.borrow_mut().get_default_font_with_size_and_dpi(s, d)
    }

    fn get_font<T>(&self, family_name: T) -> Option<Self::FontInstance>
    where
        T: AsRef<str>
    {
        let mut borrow = self.borrow_mut();
        let font_instance_id = FontInstanceId::from_family_str(family_name, DEFAULT_FONT_SIZE, DEFAULT_FONT_DPI);
        borrow.get_or_insert_font(font_instance_id)
    }

    fn get_font_with_size<T>(&self, family_name: T, size: u32) -> Option<Self::FontInstance>
    where
        T: AsRef<str>
    {
        let mut borrow = self.borrow_mut();
        let font_instance_id = FontInstanceId::from_family_str(family_name, size, DEFAULT_FONT_DPI);
        borrow.get_or_insert_font(font_instance_id)
    }

    fn get_font_with_size_and_dpi<T>(&self, family_name: T, size: u32, dpi: u32) -> Option<Self::FontInstance>
    where
        T: AsRef<str>
    {
        let mut borrow = self.borrow_mut();
        let font_instance_id = FontInstanceId::from_family_str(family_name, size, dpi);
        borrow.get_or_insert_font(font_instance_id)
    }

    fn shape_text_h<T>(&self, instance: &Self::FontInstance, text: T) -> Option<Self::Glyphs>
    where
        T: AsRef<str>
    {
        self.borrow().shape_text_h(instance, text).ok()
    }

    fn shape_text_v<T>(&self, instance: &Self::FontInstance, text: T) -> Option<Self::Glyphs>
    where
        T: AsRef<str>
    {
        self.borrow().shape_text_v(instance, text).ok()
    }

    fn take_resource_updates(&mut self) -> Self::ResourceUpdates {
        self.borrow_mut().api.take_resource_updates()
    }
}

#[derive(Debug, PartialEq)]
pub struct FontCache<A: TFontKeysAPI> {
    api: A,
    context: FontContext,
    instances: FnvHashMap<FontInstanceId, RcFontInstance<A>>,
    default_font: Option<FontInstanceId>
}

impl<A> FontCache<A>
where
    A: TFontKeysAPI
{
    pub fn new(api: A) -> Result<Self> {
        Ok(FontCache {
            api,
            context: FontContext::new()?,
            instances: FnvHashMap::default(),
            default_font: None
        })
    }

    pub fn add_raw<T>(&mut self, font_id: FontId, bytes: T, face_index: usize) -> Result<()>
    where
        T: Into<Rc<Vec<u8>>>
    {
        let encoded = EncodedFont::from_bytes(bytes)?;
        self.add_font(font_id, &encoded, face_index)
    }

    pub fn add_font<E>(&mut self, font_id: FontId, encoded: &E, face_index: usize) -> Result<()>
    where
        E: TEncodedFont
    {
        let decoded = DecodedFont::from_encoded_font(encoded, face_index);
        self.context.add_face(font_id, &decoded.bytes, face_index)?;

        let family_name = self.context.get_family_name(font_id)?;
        let size = DEFAULT_FONT_SIZE;
        let dpi = DEFAULT_FONT_DPI;
        let font_instance_id = FontInstanceId::from_family_str(family_name, size, dpi);
        self.default_font.get_or_insert(font_instance_id);

        match self.instances.entry(font_instance_id) {
            Entry::Occupied(_) => {
                Err(FontError::FontInstanceAlreadyAdded)?;
            }
            Entry::Vacant(e) => {
                let instance_data = FontInstanceResourceData::new(size, dpi);
                let external_key = self.api.add_font(encoded.info(), decoded.info());
                let external_instance_key = self.api.add_font_instance(external_key, instance_data);
                e.insert(FontInstance::new(
                    font_id,
                    size,
                    dpi,
                    external_key,
                    external_instance_key
                ));
            }
        }

        Ok(())
    }

    pub fn get_family_name_for_id(&self, id: FontId) -> Result<String> {
        self.context.get_family_name(id).map(String::from)
    }

    pub fn set_default_font<T>(&mut self, family_name: T, size: u32, dpi: u32)
    where
        T: AsRef<str>
    {
        let font_instance_id = FontInstanceId::from_family_str(family_name, size, dpi);
        self.default_font = Some(font_instance_id);
    }

    pub fn get_default_font(&self) -> Option<RcFontInstance<A>> {
        let font_instance_id = self.default_font?;
        self.instances.get(&font_instance_id).map(Rc::clone)
    }

    pub fn get_default_font_with_size(&mut self, size: u32) -> Option<RcFontInstance<A>> {
        let font_instance_id = self.default_font?.resize(size);
        self.get_or_insert_font(font_instance_id)
    }

    pub fn get_default_font_with_size_and_dpi(&mut self, size: u32, dpi: u32) -> Option<RcFontInstance<A>> {
        let font_instance_id = self.default_font?.resize_dpi(size, dpi);
        self.get_or_insert_font(font_instance_id)
    }

    pub fn get_or_insert_font(&mut self, font_instance_id: FontInstanceId) -> Option<RcFontInstance<A>> {
        let (font_id, external_key) = {
            let font_instance_id = font_instance_id.resize_dpi(DEFAULT_FONT_SIZE, DEFAULT_FONT_DPI);
            let instance = self.instances.get(&font_instance_id)?;
            (instance.font_id(), instance.external_key())
        };

        Some(match self.instances.entry(font_instance_id) {
            Entry::Occupied(e) => Rc::clone(e.get()),
            Entry::Vacant(e) => {
                let size = font_instance_id.size;
                let dpi = font_instance_id.dpi;
                let instance_data = FontInstanceResourceData::new(size, dpi);
                let external_instance_key = self.api.add_font_instance(external_key, instance_data);
                Rc::clone(e.insert(FontInstance::new(
                    font_id,
                    size,
                    dpi,
                    external_key,
                    external_instance_key
                )))
            }
        })
    }

    pub fn get_bytes(&self, font_id: FontId) -> Result<Rc<Vec<u8>>> {
        self.context.get_bytes(font_id)
    }

    pub fn get_face_index(&self, font_id: FontId) -> Result<usize> {
        self.context.get_face_index(font_id)
    }

    pub fn get_family_name(&self, font_id: FontId) -> Result<&str> {
        self.context.get_family_name(font_id)
    }

    pub fn get_glyph_index(&self, instance: FontInstanceRef<A>, c: char) -> Result<u32> {
        self.context.get_glyph_index(instance, c)
    }

    pub fn get_glyph_dimensions(&self, instance: FontInstanceRef<A>, c: char) -> Result<GlyphDimensions> {
        self.context.get_glyph_dimensions(instance, c)
    }

    pub fn get_global_size_metrics(&self, instance: FontInstanceRef<A>) -> Result<FontSizeMetrics> {
        self.context.get_global_size_metrics(instance)
    }

    pub fn shape_text_h<T>(
        &self,
        instance: FontInstanceRef<A>,
        text: T
    ) -> Result<GlyphStore<A::FontKey, A::FontInstanceKey, A::GlyphInstance>>
    where
        T: AsRef<str>
    {
        self.context.shape_text_h(instance, text)
    }

    pub fn shape_text_v<T>(
        &self,
        instance: FontInstanceRef<A>,
        text: T
    ) -> Result<GlyphStore<A::FontKey, A::FontInstanceKey, A::GlyphInstance>>
    where
        T: AsRef<str>
    {
        self.context.shape_text_v(instance, text)
    }
}

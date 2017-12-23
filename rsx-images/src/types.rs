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
use std::sync::Arc;

use fnv::{FnvHashMap, FnvHasher};
use rsx_shared::traits::{TDimensionsInfo, TEncodedImage, TImageCache, TImageKeysAPI, TMediaKey};
use uuid::Uuid;

use error::{ImageError, Result};

pub use decoded::DecodedImage;
pub use encoded::EncodedImage;
pub use rsx_shared::types::{ImageEncodedData, ImageEncodingFormat, ImagePixelFormat, ImageResourceData};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImageId(u64);

impl ImageId {
    pub fn new<P>(src: P) -> Self
    where
        P: AsRef<str>
    {
        let mut hasher = FnvHasher::default();
        hasher.write(src.as_ref().as_bytes());
        ImageId(hasher.finish())
    }

    pub fn uuid() -> Self {
        let mut hasher = FnvHasher::default();
        Uuid::new_v4().hash(&mut hasher);
        ImageId(hasher.finish())
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct ImageDimensionsInfo<ImageKey> {
    image_key: ImageKey,
    size: (u32, u32)
}

impl<ImageKey> TDimensionsInfo for ImageDimensionsInfo<ImageKey>
where
    ImageKey: TMediaKey
{
    type ResourceKey = ImageKey;

    fn resource_key(&self) -> Self::ResourceKey {
        self.image_key
    }

    fn width(&self) -> u32 {
        self.size.0
    }

    fn height(&self) -> u32 {
        self.size.1
    }
}

#[derive(Debug, PartialEq)]
pub struct Image<ImageKey> {
    format: ImagePixelFormat,
    size: (u32, u32),
    pixels: Arc<Vec<u8>>,
    external_key: ImageKey
}

impl<ImageKey> Image<ImageKey> {
    pub fn new(format: ImagePixelFormat, size: (u32, u32), pixels: Arc<Vec<u8>>, external_key: ImageKey) -> Rc<Self> {
        Rc::new(Image {
            format,
            size,
            pixels,
            external_key
        })
    }

    pub fn format(&self) -> ImagePixelFormat {
        self.format
    }

    pub fn width(&self) -> u32 {
        self.size.0
    }

    pub fn height(&self) -> u32 {
        self.size.1
    }

    pub fn pixels(&self) -> Arc<Vec<u8>> {
        Arc::clone(&self.pixels)
    }

    pub fn external_key(&self) -> ImageKey
    where
        ImageKey: Copy
    {
        self.external_key
    }

    pub fn to_dimensions_info(&self) -> ImageDimensionsInfo<ImageKey>
    where
        ImageKey: Copy
    {
        ImageDimensionsInfo {
            image_key: self.external_key,
            size: self.size
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SharedImages<A: TImageKeysAPI>(Rc<RefCell<ImageCache<A>>>);

impl<A> From<ImageCache<A>> for SharedImages<A>
where
    A: TImageKeysAPI
{
    fn from(value: ImageCache<A>) -> Self {
        SharedImages(Rc::new(RefCell::new(value)))
    }
}

impl<A> Clone for SharedImages<A>
where
    A: TImageKeysAPI
{
    fn clone(&self) -> Self {
        SharedImages(Rc::clone(&self.0))
    }
}

impl<A> Deref for SharedImages<A>
where
    A: TImageKeysAPI
{
    type Target = RefCell<ImageCache<A>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<A> TImageCache for SharedImages<A>
where
    A: TImageKeysAPI + 'static
{
    type Image = Rc<Image<A::ImageKey>>;
    type ImageId = ImageId;
    type ResourceUpdates = A::ResourceUpdates;
    type Dimensions = ImageDimensionsInfo<A::ImageKey>;

    fn add_raw<P, T>(&mut self, src: P, bytes: T) -> Option<()>
    where
        P: AsRef<str>,
        T: Into<Rc<Vec<u8>>>
    {
        let id = ImageId::new(src);
        self.borrow_mut().add_raw(id, bytes).ok()
    }

    fn add_image<P, E>(&mut self, src: P, encoded: &E) -> Option<()>
    where
        P: AsRef<str>,
        E: TEncodedImage
    {
        let id = ImageId::new(src);
        self.borrow_mut().add_image(id, encoded).ok()
    }

    fn add_image_with_id<E>(&mut self, id: Self::ImageId, encoded: &E) -> Option<()>
    where
        E: TEncodedImage
    {
        self.borrow_mut().add_image(id, encoded).ok()
    }

    fn get_image<P>(&self, src: P) -> Option<Self::Image>
    where
        P: AsRef<str>
    {
        self.borrow().get_image(src)
    }

    fn measure_image<P>(&self, src: P) -> Option<Self::Dimensions>
    where
        P: AsRef<str>
    {
        self.borrow().measure_image(src)
    }

    fn take_resource_updates(&mut self) -> Self::ResourceUpdates {
        self.borrow_mut().api.take_resource_updates()
    }
}

#[derive(Debug, PartialEq)]
pub struct ImageCache<A: TImageKeysAPI> {
    api: A,
    images: FnvHashMap<ImageId, Rc<Image<A::ImageKey>>>
}

impl<A> ImageCache<A>
where
    A: TImageKeysAPI
{
    pub fn new(api: A) -> Result<Self> {
        Ok(ImageCache {
            api,
            images: FnvHashMap::default()
        })
    }

    pub fn add_raw<T>(&mut self, image_id: ImageId, bytes: T) -> Result<()>
    where
        T: Into<Rc<Vec<u8>>>
    {
        let encoded = EncodedImage::from_bytes(bytes)?;
        self.add_image(image_id, &encoded)
    }

    pub fn add_image<E>(&mut self, image_id: ImageId, encoded: &E) -> Result<()>
    where
        E: TEncodedImage
    {
        match self.images.entry(image_id) {
            Entry::Occupied(_) => {
                Err(ImageError::ImageAlreadyAdded)?;
            }
            Entry::Vacant(e) => {
                let decoded = DecodedImage::from_encoded_image(encoded)?;
                let external_key = self.api.add_image(encoded.info(), decoded.info());
                e.insert(Image::new(
                    decoded.format,
                    decoded.size,
                    decoded.pixels,
                    external_key
                ));
            }
        }

        Ok(())
    }

    pub fn get_image<P>(&self, src: P) -> Option<Rc<Image<A::ImageKey>>>
    where
        P: AsRef<str>
    {
        self.images.get(&ImageId::new(src)).map(Rc::clone)
    }

    pub fn measure_image<P>(&self, src: P) -> Option<ImageDimensionsInfo<A::ImageKey>>
    where
        P: AsRef<str>
    {
        Some(self.images.get(&ImageId::new(src))?.to_dimensions_info())
    }
}

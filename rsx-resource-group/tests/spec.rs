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

extern crate base64_util;
extern crate rsx_resource_updates;
extern crate rsx_resources;
extern crate rsx_shared;

use std::rc::Rc;

use rsx_resource_updates::types::*;
use rsx_resources::files::types::*;
use rsx_resources::fonts::types::*;
use rsx_resources::images::types::*;
use rsx_shared::traits::*;

type ImageKeysAPI = DefaultImageKeysAPI;
type FontKeysAPI = DefaultFontKeysAPI;

#[test]
fn test_encode_decode() {
    let bytes = include_bytes!("fixtures/Quantum.png");
    let format = EncodedImage::guess_format(bytes).unwrap();

    assert_eq!(format, ImageEncodingFormat::PNG);

    let encoded = base64_util::to_image_data_uri(format.as_ref(), bytes);
    let decoded = base64_util::from_data_uri(&encoded).unwrap();

    assert_eq!(&decoded[..], &bytes[..]);
}

#[test]
fn test_image_cache_1() {
    let mut files_cache = FileCache::new().unwrap();

    let image_path = "tests/fixtures/Quantum.png";
    assert!(files_cache.add_file(image_path).is_ok());

    let image_keys = ImageKeysAPI::new(());
    let mut images_cache = ImageCache::new(image_keys).unwrap();

    let image_id = ImageId::new("Quantum");
    let image_bytes = files_cache.get_file(image_path).unwrap();

    assert!(images_cache.get_image("Quantum").is_none());

    assert!(
        images_cache
            .add_raw(image_id, Rc::clone(&image_bytes))
            .is_ok()
    );
    assert!(
        images_cache
            .add_raw(image_id, Rc::clone(&image_bytes))
            .is_err()
    );
    assert!(
        images_cache
            .add_raw(image_id, Rc::clone(&image_bytes))
            .is_err()
    );

    let image = images_cache.get_image("Quantum").unwrap();
    assert_eq!(image.width(), 512);
    assert_eq!(image.height(), 529);
    assert_eq!(image.format(), ImagePixelFormat::RGBA(8));
}

#[test]
fn test_fonts_cache_1() {
    let mut files_cache = FileCache::new().unwrap();

    let font_path = "tests/fixtures/FreeSans.ttf";
    assert!(files_cache.add_file(font_path).is_ok());

    let font_keys = FontKeysAPI::new(());
    let mut fonts_cache = FontCache::new(font_keys).unwrap();

    let font_id = FontId::new("FreeSans");
    let font_bytes = files_cache.get_file(font_path).unwrap();

    assert!(
        fonts_cache
            .get_or_insert_font(FontInstanceId::from_family_str("FreeSans", 16, 72))
            .is_none()
    );

    assert!(
        fonts_cache
            .add_raw(font_id, Rc::clone(&font_bytes), 0)
            .is_ok()
    );
    assert!(
        fonts_cache
            .add_raw(font_id, Rc::clone(&font_bytes), 0)
            .is_err()
    );
    assert!(
        fonts_cache
            .add_raw(font_id, Rc::clone(&font_bytes), 0)
            .is_err()
    );

    assert!(
        fonts_cache
            .get_or_insert_font(FontInstanceId::from_family_str("FreeSans", 16, 72))
            .is_some()
    );
}

#[test]
fn test_fonts_cache_2() {
    let mut files_cache = FileCache::new().unwrap();

    let font_path_1 = "tests/fixtures/FreeSans.ttf";
    assert!(files_cache.add_file(font_path_1).is_ok());

    let font_path_2 = "tests/fixtures/FiraMono-Regular.ttf";
    assert!(files_cache.add_file(font_path_2).is_ok());

    let font_keys = FontKeysAPI::new(());
    let mut fonts_cache = FontCache::new(font_keys).unwrap();

    let font_id_1 = FontId::new("FreeSans");
    let font_id_2 = FontId::new("FiraMono");
    let font_bytes_1 = files_cache.get_file(font_path_1).unwrap();
    let font_bytes_2 = files_cache.get_file(font_path_2).unwrap();

    assert!(
        fonts_cache
            .get_or_insert_font(FontInstanceId::from_family_str("FreeSans", 16, 72))
            .is_none()
    );
    assert!(
        fonts_cache
            .get_or_insert_font(FontInstanceId::from_family_str("Fira Mono", 16, 72))
            .is_none()
    );

    assert!(
        fonts_cache
            .add_raw(font_id_1, Rc::clone(&font_bytes_1), 0)
            .is_ok()
    );
    assert!(
        fonts_cache
            .add_raw(font_id_1, Rc::clone(&font_bytes_1), 0)
            .is_err()
    );
    assert!(
        fonts_cache
            .add_raw(font_id_1, Rc::clone(&font_bytes_1), 0)
            .is_err()
    );

    assert!(
        fonts_cache
            .add_raw(font_id_2, Rc::clone(&font_bytes_2), 0)
            .is_ok()
    );
    assert!(
        fonts_cache
            .add_raw(font_id_2, Rc::clone(&font_bytes_2), 0)
            .is_err()
    );
    assert!(
        fonts_cache
            .add_raw(font_id_2, Rc::clone(&font_bytes_2), 0)
            .is_err()
    );

    assert!(
        fonts_cache
            .get_or_insert_font(FontInstanceId::from_family_str("FreeSans", 16, 72))
            .is_some()
    );
    assert!(
        fonts_cache
            .get_or_insert_font(FontInstanceId::from_family_str("Fira Mono", 16, 72))
            .is_some()
    );
}

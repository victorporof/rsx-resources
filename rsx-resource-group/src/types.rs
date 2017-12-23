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

use rsx_shared::traits::{TFontKeysAPI, TImageKeysAPI, TResourceGroup};

use files::types::SharedFiles;
use fonts::types::SharedFonts;
use images::types::SharedImages;

#[derive(Debug, PartialEq)]
pub struct ResourceGroup<ImageKeysAPI: TImageKeysAPI, FontKeysAPI: TFontKeysAPI> {
    files: SharedFiles,
    images: SharedImages<ImageKeysAPI>,
    fonts: SharedFonts<FontKeysAPI>
}

impl<ImageKeysAPI, FontKeysAPI> TResourceGroup for ResourceGroup<ImageKeysAPI, FontKeysAPI>
where
    ImageKeysAPI: TImageKeysAPI + 'static,
    FontKeysAPI: TFontKeysAPI + 'static
{
    type Files = SharedFiles;
    type Images = SharedImages<ImageKeysAPI>;
    type Fonts = SharedFonts<FontKeysAPI>;

    fn files(&self) -> Self::Files {
        Self::Files::clone(&self.files)
    }

    fn images(&self) -> Self::Images {
        Self::Images::clone(&self.images)
    }

    fn fonts(&self) -> Self::Fonts {
        Self::Fonts::clone(&self.fonts)
    }
}

impl<ImageKeysAPI, FontKeysAPI> ResourceGroup<ImageKeysAPI, FontKeysAPI>
where
    ImageKeysAPI: TImageKeysAPI,
    FontKeysAPI: TFontKeysAPI
{
    pub fn new<FileCache, ImagesCache, FontCache>(files: FileCache, images: ImagesCache, fonts: FontCache) -> Self
    where
        FileCache: Into<SharedFiles>,
        ImagesCache: Into<SharedImages<ImageKeysAPI>>,
        FontCache: Into<SharedFonts<FontKeysAPI>>
    {
        ResourceGroup {
            files: files.into(),
            images: images.into(),
            fonts: fonts.into()
        }
    }
}

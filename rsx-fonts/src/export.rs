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

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use types::GlyphsArray;

impl<GlyphInstance> Serialize for GlyphsArray<GlyphInstance>
where
    GlyphInstance: Serialize
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        serializer.serialize_some(&self.0)
    }
}

impl<'de, GlyphInstance> Deserialize<'de> for GlyphsArray<GlyphInstance> {
    fn deserialize<D>(_: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        unimplemented!()
    }
}

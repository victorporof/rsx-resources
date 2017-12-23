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

extern crate base64;

pub fn to_image_data_uri(format: &str, bytes: &[u8]) -> String {
    let encoded = base64::encode(bytes);
    format!("data:image/{};base64,{}", format, encoded)
}

pub fn to_font_data_uri(bytes: &[u8]) -> String {
    let encoded = base64::encode(bytes);
    format!("data:application/x-font-woff;base64,{}", encoded)
}

pub fn from_data_uri(data_uri: &str) -> Result<Vec<u8>, base64::DecodeError> {
    let start = data_uri.find("base64,").unwrap_or(0) + 7;
    base64::decode(&data_uri.as_bytes()[start..])
}

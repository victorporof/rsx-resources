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

#![cfg_attr(feature = "cargo-clippy", allow(match_ref_pats, type_complexity))]
#![feature(integer_atomics)]

extern crate base64_util;
#[macro_use]
extern crate bitflags;
extern crate fnv;
extern crate freetype;
extern crate rsx_resource_updates;
extern crate rsx_shared;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate uuid;

pub mod error;
pub mod types;
pub mod encoded;
pub mod decoded;
pub mod export;

mod font_context;
mod font_face;

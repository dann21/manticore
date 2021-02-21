// Copyright lowRISC contributors.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

// Adapted from tock-on-titan.git:/shared-lib/spiutils/src/protocol/payload.rs

// Copyright 2020 lowRISC contributors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

//! SPI flash protocol payload.

use crate::io::Read;
use crate::io::Write;
use crate::mem::Arena;
use crate::protocol::wire::FromWire;
use crate::protocol::wire::FromWireError;
use crate::protocol::wire::ToWire;
use crate::protocol::wire::ToWireError;
use crate::protocol::wire::WireEnum;

wire_enum! {
    /// The content type.
    pub enum SpiContentType: u8 {
        /// Unknown message type.
        Unknown = 0xff,

        /// Manticore
        Manticore = 0x01,
    }
}

/// A parsed SPI header.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SpiHeader {
    /// The content type following the SPI header.
    pub content_type: SpiContentType,

    /// The length of the content following the SPI header.
    pub content_len: u16,
}

/// The length of a payload SPI header on the wire, in bytes.
pub const SPI_HEADER_LEN: usize = 3;

impl<'a> FromWire<'a> for SpiHeader {
    fn from_wire<R: Read, A: Arena>(
        mut r: R,
        _a: &A,
    ) -> Result<Self, FromWireError> {
        let content_type_u8 = r.read_le::<u8>()?;
        let content_type = SpiContentType::from_wire_value(content_type_u8)
            .ok_or(FromWireError::OutOfRange)?;
        let content_len = r.read_le::<u16>()?;
        Ok(Self {
            content_type,
            content_len,
        })
    }
}

impl ToWire for SpiHeader {
    fn to_wire<W: Write>(&self, mut w: W) -> Result<(), ToWireError> {
        self.content_type.to_wire(&mut w)?;
        w.write_le(self.content_len)?;
        Ok(())
    }
}

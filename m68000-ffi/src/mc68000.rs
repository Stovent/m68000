// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! MC68000 FFI.

use crate::*;
use crate::fastmem::m68000_fastmem_t;

use m68000::cpu_details::Mc68000;

use paste::paste;

cinterface!(mc68000, Mc68000);
cinterface_fastmem!(mc68000, Mc68000);

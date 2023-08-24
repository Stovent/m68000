// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! MC68000 FFI.

use crate::*;

use m68000::cpu_details::Mc68000;

use paste::paste;

cinterface!(mc68000, Mc68000);

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! SCC68070 FFI.

use crate::*;

use m68000::cpu_details::Scc68070;

use paste::paste;

cinterface!(scc68070, Scc68070);

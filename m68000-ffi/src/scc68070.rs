// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! SCC68070 FFI.

use crate::*;
use crate::fastmem::m68000_fastmem_t;

use m68000::cpu_details::Scc68070;
use m68000::cached_interpreter_flat_block::CachedInterpreterFlatBlock;
use m68000::cached_interpreter_single_block::CachedInterpreterSingleBlock;
use m68000::cached_interpreter_trie_block::CachedInterpreterTrieBlock;

use paste::paste;

cinterface!(scc68070, Scc68070);
cinterface_fastmem!(scc68070, Scc68070);
cinterface_cached_interpreter!(scc68070, Scc68070, cached_interpreter_flat_block, CachedInterpreterFlatBlock);
cinterface_cached_interpreter!(scc68070, Scc68070, cached_interpreter_single_block, CachedInterpreterSingleBlock);
cinterface_cached_interpreter!(scc68070, Scc68070, cached_interpreter_trie_block, CachedInterpreterTrieBlock);

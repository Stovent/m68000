// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use m68000::MemoryAccess;

use std::panic::catch_unwind;

#[test]
fn memory_access_slice() {
    let mut mem8 = [0xFFu8; 12];
    let mut mem16 = [0xFFFFu16; 6];

    for i in 0..4 {
        let iu8 = i as u8;

        let _ = mem8.set_byte(i, iu8);
        assert_eq!(iu8, mem8.get_byte(i).unwrap());
        assert_eq!(iu8, (&mem8 as &[u8]).get_byte(i).unwrap());
        assert!(catch_unwind(|| (&mem8 as &[u8]).set_byte(0, 0)).is_err());

        let _ = mem16.set_byte(i, iu8);
        assert_eq!(iu8, mem16.get_byte(i).unwrap());
        assert_eq!(iu8, (&mem16 as &[u16]).get_byte(i).unwrap());
        assert!(catch_unwind(|| (&mem16 as &[u16]).set_byte(0, 0)).is_err());
    }

    for i in (4..8).step_by(2) {
        let iu16 = i as u16;

        let _ = mem8.set_word(i, iu16);
        assert_eq!(iu16, mem8.get_word(i).unwrap());
        assert_eq!(iu16, (&mem8 as &[u8]).get_word(i).unwrap());
        assert!(catch_unwind(|| (&mem8 as &[u8]).set_word(0, 0)).is_err());

        let _ = mem16.set_word(i, iu16);
        assert_eq!(iu16, mem16.get_word(i).unwrap());
        assert_eq!(iu16, (&mem16 as &[u16]).get_word(i).unwrap());
        assert!(catch_unwind(|| (&mem16 as &[u16]).set_word(0, 0)).is_err());
    }

    {
        let i = 8;
        let iu32 = i as u32;

        let _ = mem8.set_long(i, iu32);
        assert_eq!(iu32, mem8.get_long(i).unwrap());
        assert_eq!(iu32, (&mem8 as &[u8]).get_long(i).unwrap());
        assert!(catch_unwind(|| (&mem8 as &[u8]).set_long(0, 0)).is_err());

        let _ = mem16.set_long(i, iu32);
        assert_eq!(iu32, mem16.get_long(i).unwrap());
        assert_eq!(iu32, (&mem16 as &[u16]).get_long(i).unwrap());
        assert!(catch_unwind(|| (&mem16 as &[u16]).set_long(0, 0)).is_err());
    }

    let expected_mem8 = [0, 1, 2, 3, 0, 4, 0, 6, 0, 0, 0, 8];
    assert_eq!(expected_mem8, mem8);

    let expected_mem16 = [0x0001, 0x0203, 4, 6, 0, 8];
    assert_eq!(expected_mem16, mem16);

    assert!(catch_unwind(|| [0u8; 1].get_byte(1).unwrap()).is_err());
    assert!(catch_unwind(|| [0u8; 1].get_word(1).unwrap()).is_err());
    assert!(catch_unwind(|| [0u8; 1].get_long(1).unwrap()).is_err());
    assert!(catch_unwind(|| [0u8; 1].set_byte(1, 0).unwrap()).is_err());
    assert!(catch_unwind(|| [0u8; 1].set_word(1, 0).unwrap()).is_err());
    assert!(catch_unwind(|| [0u8; 1].set_long(1, 0).unwrap()).is_err());

    assert!(catch_unwind(|| [0u16; 1].get_byte(2).unwrap()).is_err());
    assert!(catch_unwind(|| [0u16; 1].get_word(2).unwrap()).is_err());
    assert!(catch_unwind(|| [0u16; 1].get_long(2).unwrap()).is_err());
    assert!(catch_unwind(|| [0u16; 1].set_byte(2, 0).unwrap()).is_err());
    assert!(catch_unwind(|| [0u16; 1].set_word(2, 0).unwrap()).is_err());
    assert!(catch_unwind(|| [0u16; 1].set_long(2, 0).unwrap()).is_err());
}

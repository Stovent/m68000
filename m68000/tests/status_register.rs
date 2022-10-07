// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use m68000::status_register::StatusRegister;

#[test]
fn status_register() {
    for raw in 0..=u16::MAX {
        assert_eq!(Into::<u16>::into(StatusRegister::from(raw)), raw & 0xA71F);

        let lsr = StatusRegister::from(raw);
        let mut rsr = lsr;
        rsr &= raw;
        assert_eq!(lsr, rsr);
        rsr &= 0;
        assert_eq!(0, Into::<u16>::into(rsr));
        rsr &= 0xFFFF;
        assert_eq!(0, Into::<u16>::into(rsr));

        let lsr = StatusRegister::from(raw);
        let mut rsr = lsr;
        rsr ^= raw;
        assert_eq!(0, Into::<u16>::into(rsr));
        rsr ^= 0;
        assert_eq!(0, Into::<u16>::into(rsr));
        rsr ^= 0xFFFF;
        assert_eq!(0xA71F, Into::<u16>::into(rsr));

        let lsr = StatusRegister::from(raw);
        let mut rsr = lsr;
        rsr |= raw;
        assert_eq!(lsr, rsr);
        rsr |= 0;
        assert_eq!(lsr, rsr);
        rsr |= 0xFFFF;
        assert_eq!(0xA71F, Into::<u16>::into(rsr));

        let sr = StatusRegister::from(raw);
        assert_eq!(sr.condition(0), true,  "StatusRegister::condition(0) for {raw:#X}");
        assert_eq!(sr.condition(1), false, "StatusRegister::condition(1) for {raw:#X}");
        assert_eq!(sr.condition(2), raw & 0b0101 == 0, "StatusRegister::condition(2) for {raw:#X}");
        assert_eq!(sr.condition(3), raw & 0b0101 != 0,  "StatusRegister::condition(3) for {raw:#X}");
        assert_eq!(sr.condition(4), raw & 0b0001 == 0,  "StatusRegister::condition(4) for {raw:#X}");
        assert_eq!(sr.condition(5), raw & 0b0001 != 0, "StatusRegister::condition(5) for {raw:#X}");
        assert_eq!(sr.condition(6), raw & 0b0100 == 0,  "StatusRegister::condition(6) for {raw:#X}");
        assert_eq!(sr.condition(7), raw & 0b0100 != 0, "StatusRegister::condition(7) for {raw:#X}");
        assert_eq!(sr.condition(8), raw & 0b0010 == 0, "StatusRegister::condition(8) for {raw:#X}");
        assert_eq!(sr.condition(9), raw & 0b0010 != 0,  "StatusRegister::condition(9) for {raw:#X}");
        assert_eq!(sr.condition(10), raw & 0b1000 == 0, "StatusRegister::condition(10) for {raw:#X}");
        assert_eq!(sr.condition(11), raw & 0b1000 != 0,  "StatusRegister::condition(11) for {raw:#X}");
        assert_eq!(sr.condition(12), raw & 0b1010 == 0b1010 || raw & 0b1010 == 0, "StatusRegister::condition(12) for {raw:#X}");
        assert_eq!(sr.condition(13), raw & 0b1010 == 0b1000 || raw & 0b1010 == 0b0010, "StatusRegister::condition(13) for {raw:#X}");
        assert_eq!(sr.condition(14), raw & 0b1110 == 0b1010 || raw & 0b1110 == 0, "StatusRegister::condition(14) for {raw:#X}");
        assert_eq!(sr.condition(15), raw & 0b0100 != 0 || raw & 0b1010 == 0b1000 || raw & 0b1010 == 0b0010, "StatusRegister::condition(15) for {raw:#X}");
    }
}

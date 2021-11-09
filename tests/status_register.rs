use m68000::status_register::StatusRegister;

#[test]
fn status_register() {
    let mut sr = StatusRegister {
        t: true,
        s: true,
        interrupt_mask: 7,
        x: true,
        n: true,
        z: true,
        v: true,
        c: true,
    };

    assert_eq!(Into::<u16>::into(StatusRegister::default()), 0, "StatusRegister::default()");
    assert_eq!(Into::<u16>::into(StatusRegister::from(0)), 0, "StatusRegister::from(0)");
    assert_eq!(Into::<u16>::into(sr), 0xA71F, "into::<u16>()");

    sr ^= 0b00101101_01010011;
    assert_eq!(Into::<u16>::into(sr), 0x820C, "StatusRegister xor");

    sr &= 0b00001111_01110101;
    assert_eq!(Into::<u16>::into(sr), 0x0204, "StatusRegister and");

    sr |= 0b10001000_01111001;
    assert_eq!(Into::<u16>::into(sr), 0x821D, "StatusRegister or");

    for i in 0..0xF {
        sr.set_ccr(i);
        assert_eq!(Into::<u16>::into(sr), 0x8200 | i, "StatusRegister::set_ccr for {}", i);
        assert_eq!(sr.condition(0), true,  "StatusRegister::condition(0) for {}", i);
        assert_eq!(sr.condition(1), false, "StatusRegister::condition(1) for {}", i);
        assert_eq!(sr.condition(2), i & 0b0101 == 0, "StatusRegister::condition(2) for {}", i);
        assert_eq!(sr.condition(3), i & 0b0101 != 0,  "StatusRegister::condition(3) for {}", i);
        assert_eq!(sr.condition(4), i & 0b0001 == 0,  "StatusRegister::condition(4) for {}", i);
        assert_eq!(sr.condition(5), i & 0b0001 != 0, "StatusRegister::condition(5) for {}", i);
        assert_eq!(sr.condition(6), i & 0b0100 == 0,  "StatusRegister::condition(6) for {}", i);
        assert_eq!(sr.condition(7), i & 0b0100 != 0, "StatusRegister::condition(7) for {}", i);
        assert_eq!(sr.condition(8), i & 0b0010 == 0, "StatusRegister::condition(8) for {}", i);
        assert_eq!(sr.condition(9), i & 0b0010 != 0,  "StatusRegister::condition(9) for {}", i);
        assert_eq!(sr.condition(10), i & 0b1000 == 0, "StatusRegister::condition(10) for {}", i);
        assert_eq!(sr.condition(11), i & 0b1000 != 0,  "StatusRegister::condition(11) for {}", i);
        assert_eq!(sr.condition(12), i & 0b1010 == 0b1010 || i & 0b1010 == 0, "StatusRegister::condition(12) for {}", i);
        assert_eq!(sr.condition(13), i & 0b1010 == 0b1000 || i & 0b1010 == 0b0010, "StatusRegister::condition(13) for {}", i);
        assert_eq!(sr.condition(14), i & 0b1110 == 0b1010 || i & 0b1110 == 0, "StatusRegister::condition(14) for {}", i);
        assert_eq!(sr.condition(15), i & 0b0100 != 0 || i & 0b1010 == 0b1000 || i & 0b1010 == 0b0010, "StatusRegister::condition(15) for {}", i);
    }
}

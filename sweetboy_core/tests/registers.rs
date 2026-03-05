use sweetboy_core::cpu::registers::Registers;

#[test]
fn pair_registers_roundtrip() {
    let mut r = Registers::new();

    r.set_bc(0x1234);
    assert_eq!(r.get_bc(), 0x1234);

    r.set_de(0xBEEF);
    assert_eq!(r.get_de(), 0xBEEF);

    r.set_hl(0xABCD);
    assert_eq!(r.get_hl(), 0xABCD);
}

#[test]
fn af_masks_low_nibble_of_f() {
    let mut r = Registers::new();
    r.set_af(0x12FF);
    let af = r.get_af();
    assert_eq!(af & 0x000F, 0x0000);
}

#[test]
fn flag_setters_getters_work() {
    let mut r = Registers::new();

    r.set_z(true);
    r.set_n(false);
    r.set_hc(true);
    r.set_carry(false);

    assert!(r.get_z());
    assert!(!r.get_n());
    assert!(r.get_hc());
    assert!(!r.get_carry());
}

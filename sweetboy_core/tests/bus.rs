use sweetboy_core::bus::Bus;

#[test]
fn bus_read_write_byte_roundtrip() {
    let mut bus = Bus::new();
    bus.write_byte(0xC000, 0x12);
    assert_eq!(bus.read_byte(0xC000), 0x12);
}

#[test]
fn bus_read_write_word_little_endian() {
    let mut bus = Bus::new();
    bus.write_word(0xC000, 0xBEEF);
    assert_eq!(bus.read_byte(0xC000), 0xEF); // lo
    assert_eq!(bus.read_byte(0xC001), 0xBE); // hi
    assert_eq!(bus.read_word(0xC000), 0xBEEF);
}

#[test]
fn bus_word_wraparound_reads_hi_from_0x0000() {
    let mut bus = Bus::new();
    bus.write_byte(0xFFFF, 0xAA);
    bus.write_byte(0x0000, 0xBB);
    assert_eq!(bus.read_word(0xFFFF), 0xBBAA);
}

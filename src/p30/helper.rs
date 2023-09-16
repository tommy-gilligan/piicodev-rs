use fixed::types::U20F12;

#[must_use]
pub fn millimetres_from(microseconds: u16) -> U20F12 {
    U20F12::from_num(microseconds) * U20F12::lit("0.1715")
}

#[test]
pub fn length() {
    assert_eq!(millimetres_from(10_000), U20F12::lit("1713.8672"));
}

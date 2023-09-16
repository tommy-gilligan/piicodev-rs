#![cfg(feature = "acceptance")]
mod support;

fn parse_line(line: String) -> i16 {
    let (value, unit) = line.rsplit_once(' ').unwrap();
    assert_eq!(unit, "mm");
    value.parse().unwrap()
}

#[test]
fn p7_test() {
    let mut example = support::Example::run_micropython(
        "mp/CE-PiicoDev-VL53L1X-MicroPython-Module/main.py",
        vec![
            "mp/CE-PiicoDev-Unified/min/PiicoDev_Unified.py",
            "mp/CE-PiicoDev-VL53L1X-MicroPython-Module/min/PiicoDev_VL53L1X.py",
        ],
    );
    let micropython_output: Vec<i16> = example.output().take(10).map(parse_line).collect();

    let mut example = support::Example::run("p7");
    let output: Vec<i16> = example.output().take(10).map(parse_line).collect();

    assert!(output
        .iter()
        .zip(micropython_output)
        .all(|(mm, mp_mm)| (mm - mp_mm).abs() < 10));
}

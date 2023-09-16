#![cfg(feature = "acceptance")]
mod support;

fn parse_line(line: &str) -> i16 {
    line.parse().unwrap()
}

#[test]
fn p30_read_when_available_test() {
    let mut micropython_example = support::Example::run_micropython(
        "mp/CE-PiicoDev-Ultrasonic-Rangefinder-MicroPython-Module/examples/read_when_available.py",
        vec![
            "mp/CE-PiicoDev-Unified/min/PiicoDev_Unified.py",
            "mp/CE-PiicoDev-Ultrasonic-Rangefinder-MicroPython-Module/min/PiicoDev_Ultrasonic.py",
        ],
    );

    let micropython_output: Vec<i16> = micropython_example
        .output()
        .take(10)
        .map(|l| parse_line(&l))
        .collect();

    let mut example = support::Example::run("p30_read_when_available");
    let output: Vec<i16> = example.output().take(10).map(|l| parse_line(&l)).collect();

    assert!(output
        .iter()
        .zip(micropython_output)
        .all(|(mm, mp_mm)| (mm - mp_mm).abs() < 20));
}

#![cfg(feature = "acceptance")]
mod support;

fn parse_line(line: &str) -> i16 {
    line.parse().unwrap()
}

#[test]
fn p30_simple_read_test() {
    let mut micropython_example = support::Example::run_micropython(
        "mp/CE-PiicoDev-Ultrasonic-Rangefinder-MicroPython-Module/examples/simple_read.py",
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

    let mut example = support::Example::run("p30_simple_read");
    let output: Vec<i16> = example.output().take(10).map(|l| parse_line(&l)).collect();

    assert!(output
        .iter()
        .zip(micropython_output)
        .all(|(mm, mp_mm)| (mm - mp_mm).abs() < 20));
}

#![cfg(feature = "acceptance")]
mod support;

fn parse_line(line: &str) -> f64 {
    let (value, unit) = line.rsplit_once(' ').unwrap();
    assert_eq!(unit, "°C");
    value.parse().unwrap()
}

#[test]
fn p1_test() {
    let mut micropython_example = support::Example::run_micropython(
        "mp/CE-PiicoDev-TMP117-MicroPython-Module/main.py",
        vec![
            "mp/CE-PiicoDev-Unified/min/PiicoDev_Unified.py",
            "mp/CE-PiicoDev-TMP117-MicroPython-Module/min/PiicoDev_TMP117.py",
        ],
    );
    let micropython_output: Vec<f64> = micropython_example
        .output()
        .take(10)
        .map(|l| parse_line(&l))
        .collect();

    let mut example = support::Example::run("p1");
    let output: Vec<f64> = example.output().take(10).map(|l| parse_line(&l)).collect();

    for (c, mp_c) in output.iter().zip(micropython_output) {
        assert!((c - mp_c).abs() < 2.0_f64);
    }
}

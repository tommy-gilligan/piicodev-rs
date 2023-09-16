mod support;

fn parse_line(line: String) -> f64 {
    let (value, unit) = line.rsplit_once(' ').unwrap();
    assert_eq!(unit, "lux");
    value.parse().unwrap()
}

#[test]
fn p3_test() {
    let mut example = support::Example::run_micropython(
        "mp/CE-PiicoDev-VEML6030-MicroPython-Module/main.py",
        vec![
            "mp/CE-PiicoDev-Unified/min/PiicoDev_Unified.py",
            "mp/CE-PiicoDev-VEML6030-MicroPython-Module/min/PiicoDev_VEML6030.py",
        ],
    );
    let micropython_output: Vec<f64> = example.output().take(10).map(parse_line).collect();

    let mut example = support::Example::run("p3");
    let output: Vec<f64> = example.output().take(10).map(parse_line).collect();

    assert!(output
        .iter()
        .zip(micropython_output)
        .all(|(lux, mp_lux)| (lux.log10() - mp_lux.log10()).abs() < 2.0));
}

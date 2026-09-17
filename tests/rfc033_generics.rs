use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts, check_files_in};

fn check(src: &str) -> (cohdl::pipeline::Checked, String) {
    let files = vec![("src/main.cohdl".to_string(), src.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    checked.diags.sort(&checked.sm);
    let rendered = checked.diags.render(&checked.sm);
    (checked, rendered)
}

fn netlist(src: &str) -> String {
    let files = vec![("src/main.cohdl".to_string(), src.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    let artifacts = build_artifacts(&mut checked, &LockState::default());
    checked.diags.sort(&checked.sm);
    assert!(
        !checked.diags.has_errors(),
        "clean build expected:\n{}",
        checked.diags.render(&checked.sm)
    );
    artifacts.expect("build").netlist
}

const LIB: &str = r#"
pub trait Cap { designator_prefix: "C" }
pub device CapDev { pins { A: 1 [passive], B: 2 [passive] } }
impl Cap for CapDev {}
pub footprint FP {}
pub part C100N: CapDev { primary { mfr: "m", mpn: "c", footprint: FP } }
pub device Host { pins { P: 1 [passive], Q: 2 [passive] } }
pub part HOST: Host { primary { mfr: "m", mpn: "h", footprint: FP } }
"#;

#[test]
fn fn_const_int_param_binds_and_forwards() {
    let src = format!("{LIB}
pub fn inner<const N: Int>(p: Pin) {{ inst c: [C100N; N]  net _: p, c[0..=(N - 1)].A, c[0..=(N - 1)].B }}
pub fn outer<const N: Int = 2>(p: Pin) {{ inner::<N + 1>(p) }}
design Board {{ inst h: HOST  outer(h.P)  nc: h.Q }}");
    let (c, r) = check(&src);
    assert!(!c.diags.has_errors(), "{r}");
    let n = netlist(&src);
    assert_eq!(
        n.matches("(comp (ref \"C").count(),
        3,
        "N=2 default → inner gets 3"
    );
}

#[test]
fn subdesign_const_int_param() {
    let src = format!("{LIB}
pub subdesign Bank<const N: Int> {{ ports {{ required IN: Pin }}  inst c: [C100N; N]  net _: IN, c[0..=(N - 1)].A, c[0..=(N - 1)].B }}
design Board {{ inst h: HOST  subdesign b: Bank<4> {{ IN: h.P }}  nc: h.Q }}");
    let n = netlist(&src);
    assert_eq!(n.matches("(comp (ref \"C").count(), 4);
}

#[test]
fn kind_mismatches() {
    let src = format!(
        "{LIB}
pub fn f<const N: Int>(p: Pin) {{ net _: p }}
design Board {{ inst h: HOST  f::<100nF>(h.P)  nc: h.Q }}"
    );
    assert!(
        check(&src).1.contains("E1401"),
        "unit literal for an Int parameter"
    );
    let src = format!(
        "{LIB}
pub fn g<V: Voltage>(p: Pin) {{ net _: p }}
design Board {{ inst h: HOST  g::<3>(h.P)  nc: h.Q }}"
    );
    assert!(
        check(&src).1.contains("E113"),
        "bare number for a unit parameter stays E113"
    );
    let src = format!(
        "{LIB}
pub device D<const N: Int> {{ pins {{ A: 1 [passive] }} }}
design Board {{ inst h: HOST  net _: h.P, h.Q }}"
    );
    assert!(
        check(&src).1.contains("E406"),
        "const generics on devices are rejected"
    );
    let src = format!(
        "{LIB}
pub fn f<const N: Int = 2mm>(p: Pin) {{ net _: p }}
design Board {{ inst h: HOST  net _: h.P, h.Q }}"
    );
    assert!(
        check(&src).1.contains("E406"),
        "Int default must be an integer literal"
    );
}

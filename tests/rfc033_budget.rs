use cohdl::pipeline::check_files_in;

fn check(src: &str) -> (cohdl::pipeline::Checked, String) {
    let files = vec![("src/main.cohdl".to_string(), src.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    checked.diags.sort(&checked.sm);
    let rendered = checked.diags.render(&checked.sm);
    (checked, rendered)
}

const LIB: &str = r#"
pub trait Led { designator_prefix: "D" }
pub device LedDev { pins { A: 1 [passive], B: 2 [passive] } }
impl Led for LedDev {}
pub device Host { pins { P: 1 [passive], Q: 2 [passive] } }
pub footprint FP {}
pub part LED: LedDev { primary { mfr: "m", mpn: "led", footprint: FP } }
pub part HOST: Host { primary { mfr: "m", mpn: "host", footprint: FP } }
"#;

// RFC §9's worked example: 2 iterations × 8 work items — passes.
#[test]
fn rfc_worked_example_2x8() {
    let src = format!(
        "{LIB}
design B {{
    inst led: LED
    for links: n in 0..2 {{
        nc: led.B
    }}
    net _: led.A
}}"
    );
    let (chk, r) = check(&src);
    assert!(!chk.diags.has_errors(), "{r}");
}

#[test]
fn iteration_budget_exact_boundaries() {
    // Every ENTERED iteration counts cumulatively (outer + inner): 1000
    // outer + 1000×99 inner = exactly 100,000 — passes at the limit.
    let src = format!(
        "{LIB}
design B {{
    inst host: HOST
    inst led: LED
    for a: i in 0..1000 {{
        for b: j in 0..99 {{
            nc: led.B
        }}
    }}
    net _: host.P, host.Q, led.A
}}"
    );
    let (chk, r) = check(&src);
    assert!(
        !chk.diags.has_errors(),
        "100,000 iterations is the limit:\n{r}"
    );

    // 1001 × 100 = 100,100 — the 100,001st iteration trips E1405, once.
    let src = format!(
        "{LIB}
design B {{
    inst host: HOST
    inst led: LED
    for a: i in 0..1001 {{
        for b: j in 0..100 {{
            nc: led.B
        }}
    }}
    net _: host.P, host.Q, led.A
}}"
    );
    let r = check(&src).1;
    assert!(r.contains("E1405"), "{r}");
    assert!(
        r.contains("100,001") || r.contains("100001"),
        "names the failing iteration count:\n{r}"
    );
    assert_eq!(
        r.matches("error[E1405]").count(),
        1,
        "E1405 reports exactly once:\n{r}"
    );
}

#[test]
fn frame_depth_limit_is_64() {
    // 64 nested loops, each iterating once — the frame stack peaks at 64.
    let mut inner = String::new();
    for d in 0..64 {
        inner.push_str(&format!("for d{d}: i{d} in 0..1 {{"));
    }
    inner.push_str("net _: led.A");
    for _ in 0..64 {
        inner.push_str("}");
    }
    let src = format!(
        "{LIB}
design B {{
    inst led: LED
    {inner}
    nc: led.B
}}"
    );
    let (chk, r) = check(&src);
    assert!(!chk.diags.has_errors(), "64 frames is the limit:\n{r}");

    // 65 nested loops trip the depth check.
    let mut inner = String::new();
    for d in 0..65 {
        inner.push_str(&format!("for d{d}: i{d} in 0..1 {{"));
    }
    inner.push_str("net _: led.A");
    for _ in 0..65 {
        inner.push_str("}");
    }
    let src = format!(
        "{LIB}
design B {{
    inst led: LED
    {inner}
    net _: led.A
}}"
    );
    let r = check(&src).1;
    assert!(r.contains("E1405") && r.contains("frames"), "{r}");
    assert_eq!(r.matches("error[E1405]").count(), 1, "once:\n{r}");
}

#[test]
fn legacy_graph_is_not_metered() {
    // No M2 syntax anywhere in the reachable graph: no metering, hence no
    // E1405 even at silly literal scales (10 literal instances here).
    let src = format!("{LIB}
design B {{
    inst a0: HOST
    inst a1: HOST
    inst a2: HOST
    inst a3: HOST
    inst a4: HOST
    inst a5: HOST
    inst a6: HOST
    inst a7: HOST
    inst a8: HOST
    inst a9: HOST
    net _: a0.P, a0.Q, a1.P, a1.Q, a2.P, a2.Q, a3.P, a3.Q, a4.P, a4.Q, a5.P, a5.Q, a6.P, a6.Q, a7.P, a7.Q, a8.P, a8.Q, a9.P, a9.Q
}}");
    let (chk, r) = check(&src);
    assert!(!chk.diags.has_errors(), "{r}");
    assert!(!r.contains("E1405"));
}

#[test]
fn uncalled_m2_helper_does_not_activate_metering() {
    // The helper has M2 syntax but is never referenced: no metering.
    let src = format!(
        "{LIB}
pub fn helper(p: Pin) {{ for looped: n in 0..1000001 {{ nc: p }} }}
design B {{
    inst host: HOST
    net _: host.P, host.Q
}}"
    );
    let (chk, r) = check(&src);
    assert!(
        !chk.diags.has_errors(),
        "uncalled M2 fn must not meter:\n{r}"
    );
    assert!(!r.contains("E1405"));
}

#[test]
fn work_item_budget_trips_once() {
    // Referencing the M2 helper ACTIVATES metering; the cheap overflow is a
    // literal array of 1,000,001 elements charged 1 each before allocation.
    let src = format!(
        "{LIB}
pub fn helper<const K: Int>(p: Pin) {{ nc: p }}
design B {{
    inst host: HOST
    inst huge: [LED; 1000001]
    for act: n in 0..1 {{ helper::<3>(host.P) }}
    nc: host.Q
    nc: huge[0].A
}}"
    );
    let r = check(&src).1;
    assert!(r.contains("E1405"), "1,000,001 work items trips:\n{r}");
    assert_eq!(r.matches("error[E1405]").count(), 1, "exactly once:\n{r}");
}

use cohdl::fmt::format_source;

const SRC: &str = "pub device Dev { pins { A: 1 [passive], B: 2 [passive] } }
design Board {
    const N: Int = (2+1)*2
    inst d: [Dev; N]
    for links: n in 0..N-1 {
        net _: d[n].B, d[n+1].A
    }
    net IN: d[0].A
    nc: d[N - 1].B
    layout {
        for grid: n in 0..d.len {
            place d[n] at (10mm+n*4mm, -1.5mm) rotate 90*n
        }
    }
}
";

#[test]
fn canonical_and_idempotent() {
    let once = format_source("main.cohdl", SRC).unwrap();
    assert!(once.contains("const N: Int = (2 + 1) * 2"));
    assert!(once.contains("for links: n in 0..N - 1 {"));
    assert!(once.contains("net _: d[n].B, d[n + 1].A"));
    assert!(once.contains("place d[n] at (10mm + n * 4mm, -1.5mm) rotate 90 * n"));
    let twice = format_source("main.cohdl", &once).unwrap();
    assert_eq!(once, twice);
}

#[test]
fn parentheses_are_never_dropped() {
    let src = "design Board { const A: Int = (1 + 2) * 3 }\n";
    let out = format_source("main.cohdl", src).unwrap();
    assert!(out.contains("(1 + 2) * 3"));
}

#[test]
fn rotate_zero_is_never_printed() {
    let src = "pub device Dev { pins { A: 1 [passive], B: 2 [passive] } }
design Board {
    inst d: Dev
    net _: d.A, d.B
    layout { place d at (0mm, 0mm) rotate 0 }
}
";
    let out = format_source("main.cohdl", src).unwrap();
    assert!(out.contains("place d at (0mm, 0mm)"), "{}", out);
    assert!(
        !out.contains("rotate"),
        "`rotate 0` must not print:\n{}",
        out
    );
}

#[test]
fn nested_layout_loop_and_const_round_trip() {
    let src = "pub device Dev { pins { A: 1 [passive], B: 2 [passive] } }
design Board {
    inst d: [Dev; 2]
    layout {
        const P: Length = 2mm
        for grid: n in 0..2 {
            const Q: Int = n + 1
            place d[n] at (P * Q, -1.5mm) rotate 90 * Q
            for inner: m in 0..Q {
                place d[m] at (1mm, 1mm)
            }
        }
    }
}
";
    let once = format_source("main.cohdl", src).unwrap();
    assert!(once.contains("const P: Length = 2mm"), "{}", once);
    assert!(once.contains("for grid: n in 0..2 {"), "{}", once);
    assert!(once.contains("const Q: Int = n + 1"), "{}", once);
    assert!(
        once.contains("place d[n] at (P * Q, -1.5mm) rotate 90 * Q"),
        "{}",
        once
    );
    assert!(once.contains("for inner: m in 0..Q {"), "{}", once);
    let twice = format_source("main.cohdl", &once).unwrap();
    assert_eq!(once, twice, "fmt must be idempotent");
}

#[test]
fn const_int_generic_round_trips() {
    let src = "pub device Dev { pins { A: 1 [passive], B: 2 [passive] } }
fn bank<const N: Int = 2, L: Length>(p: Pin) { net _: p }
design Board {
    inst d: Dev
    net _: d.A, d.B
}
";
    let once = format_source("main.cohdl", src).unwrap();
    assert!(once.contains("<const N: Int = 2, L: Length>"), "{}", once);
    let twice = format_source("main.cohdl", &once).unwrap();
    assert_eq!(once, twice);
}

#[test]
fn unary_minus_is_tight_and_literals_keep_spelling() {
    let src = "design Board { const A: Length = - 1.00mm  const B: Int = - (2 + 1) }\n";
    let out = format_source("main.cohdl", src).unwrap();
    // `- 1.00mm` (space) is unary-neg on a literal: unary is tight, the
    // literal's own text keeps its spelling.
    assert!(out.contains("-1.00mm"), "{}", out);
    assert!(out.contains("-(2 + 1)"), "{}", out);
    let twice = format_source("main.cohdl", &out).unwrap();
    assert_eq!(out, twice);
}

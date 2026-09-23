//! RFC-033 §Scenarios — the three normative acceptance shapes, transcribed
//! onto synthetic devices/parts (the RFC's scenario devices are fixtures of
//! the proposal tree, not shipped parts; these synthetic twins carry the
//! same pins/kinds so every count and override assertion is the RFC's own).

use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts, check_files_in};

fn check(src: &str) -> cohdl::pipeline::Checked {
    let files = vec![("src/main.cohdl".to_string(), src.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    checked.diags.sort(&checked.sm);
    assert!(
        !checked.diags.has_errors(),
        "clean check expected:\n{}",
        checked.diags.render(&checked.sm)
    );
    checked
}

fn build(src: &str, prior: &LockState) -> cohdl::pipeline::BuildArtifacts {
    let mut checked = check(src);
    build_artifacts(&mut checked, prior).expect("build")
}

const LIB: &str = r#"
pub trait LedPfx { designator_prefix: "D" }
pub device AddressableLED { pins { VDD: 1 [power_in], GND: 2 [power_in], DIN: 3 [input], DOUT: 4 [output] } }
impl LedPfx for AddressableLED {}
pub device Host { pins { V5: 1 [power_out], GND: 2 [power_in], DATA: 3 [output] } }
pub trait ResPfx { designator_prefix: "R" }
pub device SeriesR { pins { A: 1 [passive], B: 2 [passive] } }
impl ResPfx for SeriesR {}
pub trait CapPfx { designator_prefix: "C" }
pub device ShuntC { pins { A: 1 [passive], B: 2 [passive] } }
impl CapPfx for ShuntC {}
pub device SignalSource { pins { OUT: 1 [output] } }
pub device SignalSink { pins { IN: 1 [input] } }
pub device Ground { pins { GND: 1 [power_out] } }
pub footprint FP {}
pub part LEDP: AddressableLED { primary { mfr: "m", mpn: "led", footprint: FP } }
pub part HOSTP: Host { primary { mfr: "m", mpn: "host", footprint: FP } }
pub part RP: SeriesR { primary { mfr: "m", mpn: "r", footprint: FP } }
pub part CP: ShuntC { primary { mfr: "m", mpn: "c", footprint: FP } }
pub part SRCP: SignalSource { primary { mfr: "m", mpn: "src", footprint: FP } }
pub part SNKP: SignalSink { primary { mfr: "m", mpn: "snk", footprint: FP } }
pub part GNDP: Ground { primary { mfr: "m", mpn: "gnd", footprint: FP } }
"#;

fn led_chain(n: i64) -> String {
    format!(
        "{LIB}
design Chain {{
    const N: Int = {n}
    inst host: HOSTP
    inst leds: [LEDP; N]
    net VCC [5V]: host.V5, leds[0..=(leds.len - 1)].VDD
    net GND [gnd]: host.GND, leds[0..=(leds.len - 1)].GND
    net DATA: host.DATA, leds[0].DIN
    for links: n in 0..(leds.len - 1) {{
        net _: leds[n].DOUT, leds[n + 1].DIN
    }}
    nc: leds[leds.len - 1].DOUT
}}"
    )
}

#[test]
fn led_chain_neighbor_nets_0_1_9() {
    for (n, links) in [(1, 0), (2, 1), (10, 9)] {
        let art = build(&led_chain(n), &LockState::default());
        assert_eq!(
            art.netlist.matches("__for_links_").count(),
            links,
            "N={n}: {links} neighbor nets"
        );
        // instances: host + N leds
        assert_eq!(
            art.netlist.matches("(comp (ref \"").count(),
            (n + 1) as usize
        );
    }
}

fn rc_board(n: i64) -> String {
    format!(
        "{LIB}
pub subdesign RcChannel {{
    ports {{
        required IN: Pin
        required OUT: Pin
        required GND: Pin
    }}
    inst r: RP
    inst c: CP
    net _: IN, r.A
    net _: OUT, r.B, c.A
    net _: GND, c.B
    layout {{
        place r at (0mm, 0mm)
        place c at (3mm, 0mm)
    }}
}}

design FilterBoard {{
    const N: Int = {n}
    inst inputs: [SRCP; N]
    inst outputs: [SNKP; N]
    inst ground: GNDP
    subdesign channels: [RcChannel; N]

    for wiring: i in 0..channels.len {{
        net _: inputs[i].OUT, channels[i].IN
        net _: channels[i].OUT, outputs[i].IN
        net GND [gnd]: ground.GND, channels[i].GND
    }}

    layout {{
        for placement: i in 0..channels.len {{
            place channels[i] at (10mm + i * 8mm, 15mm)
        }}
        place ground at (0mm, 15mm)
        place channels[6].c at (62mm, 17mm)
    }}
}}"
    )
}

#[test]
fn rc_channels_counts_override_and_growth() {
    // N = 10: 4N+1 physical instances (10 src + 10 snk + 1 gnd + 20 rc),
    // 2N+1 net classes (2 per channel + the shared ground).
    let art = build(&rc_board(10), &LockState::default());
    assert_eq!(art.netlist.matches("(comp (ref \"").count(), 41);
    // net classes: per-channel IN/OUT nets (20) + GND = 21 distinct named
    // classes; count distinct (net (name …)) entries.
    let nets = art.netlist.matches("(net (code").count();
    assert_eq!(nets, 21, "2N+1 net classes at N=10");
    // channel 6's capacitor override lands at [62, 17].
    let layout = art.layout.as_deref().expect("layout.json");
    assert!(
        layout.contains("channels_6::c"),
        "override target:\n{layout}"
    );
    assert!(
        layout.contains("[62, 17]"),
        "override at (62mm,17mm):\n{layout}"
    );

    // N = 12 with the N=10 lock carried: everything earlier survives.
    let lock10 = art.lock.clone();
    let art12 = build(&rc_board(12), &lock10);
    let layout12 = art12.layout.as_deref().expect("layout");
    assert!(layout12.contains("channels_6::c") && layout12.contains("[62, 17]"));
    assert_eq!(art12.netlist.matches("(comp (ref \"").count(), 49);
    assert_eq!(art12.netlist.matches("(net (code").count(), 25);
}

fn bank_board() -> String {
    format!(
        "{LIB}
pub subdesign RcChannel {{
    ports {{
        required IN: Pin
        required OUT: Pin
        required GND: Pin
    }}
    inst r: RP
    inst c: CP
    net _: IN, r.A
    net _: OUT, r.B, c.A
    net _: GND, c.B
    layout {{
        place r at (0mm, 0mm)
        place c at (3mm, 0mm)
    }}
}}

pub fn join(src: Pin, dst: Pin) {{
    net _: src, dst
}}

pub subdesign FilterBank<const N: Int> {{
    ports {{
        required IN: Pin
        required GND: Pin
    }}
    inst receivers: [SNKP; N]
    subdesign channels: [RcChannel; N]
    for wiring: i in 0..N {{
        join(IN, channels[i].IN)
        join(channels[i].OUT, receivers[i].IN)
        net _: GND, channels[i].GND
    }}
    layout {{
        for placement: i in 0..N {{
            place channels[i] at (i * 8mm, 0mm)
            place receivers[i] at (i * 8mm, 10mm)
        }}
    }}
}}

design Board {{
    inst source: SRCP
    inst ground: GNDP
    subdesign bank: FilterBank<3> {{
        IN: source.OUT,
        GND: ground.GND,
    }}
    layout {{
        place bank at (10mm, 15mm)
        place bank.channels[1].c at (22mm, 17mm)
    }}
}}"
    )
}

#[test]
fn nested_filter_bank_counts_and_override() {
    // N=3 inside the bank: 3 receivers + 3×(r,c) = 9 bank-internal physical
    // instances, plus source + ground = 11 on the board.
    let art = build(&bank_board(), &LockState::default());
    assert_eq!(
        art.netlist.matches("(comp (ref \"").count(),
        11,
        "11 instances at N=3"
    );
    // Net classes: per channel IN + OUT (2 each) + the bank's shared GND =
    // 5 at N=3 (the ground merges into one class; joins collapse IN/OUT
    // pairs across the boundary).
    let nets = art.netlist.matches("(net (code").count();
    assert_eq!(nets, 5, "5 net classes at N=3");
    // The board-level override reaches through the bank boundary.
    let layout = art.layout.as_deref().expect("layout");
    assert!(
        layout.contains("bank::channels_1::c"),
        "nested override target:\n{layout}"
    );
    assert!(
        layout.contains("[22, 17]"),
        "override at (22mm,17mm):\n{layout}"
    );
}

// ---------------------------------------------------------------------------
// T1 regression: subdesign layout `for` loops keep their placements (review
// blocker #1). The frame a default is recorded against is the subdesign NODE,
// not the loop-qualified `__for_{label}_{value}` path — otherwise
// `default_placements` finds no anchor for the owner and silently drops every
// loop-authored placement.
// ---------------------------------------------------------------------------

/// One placement row by exact path, from the CHECKED IR (strict: every
/// path/coordinate/angle/side asserted directly against `DesignIr`).
fn ir_place<'a>(ir: &'a cohdl::ir::DesignIr, path: &str) -> &'a cohdl::ir::LayoutPlacement {
    ir.layout
        .placements
        .iter()
        .find(|p| p.path == path)
        .unwrap_or_else(|| {
            panic!(
                "no placement for {path}: {:?}",
                ir.layout
                    .placements
                    .iter()
                    .map(|p| &p.path)
                    .collect::<Vec<_>>()
            )
        })
}

/// The full (path, x, y, rotate, side) set, in path order, for exact-equality
/// comparisons — no contains/counting-only assertions.
fn mm(x: i128) -> i128 {
    x * 1_000_000_000_000_000
}

fn placement_set(
    ir: &cohdl::ir::DesignIr,
    keep: &[&str],
) -> Vec<(String, i128, i128, u16, cohdl::ast::PlacementSide)> {
    let mut rows: Vec<_> = ir
        .layout
        .placements
        .iter()
        .filter(|p| keep.iter().any(|k| p.path == *k))
        .map(|p| (p.path.clone(), p.at.0.femto, p.at.1.femto, p.rotate, p.side))
        .collect();
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    rows
}

#[test]
fn subdesign_layout_loop_composes_the_full_expected_set() {
    // The review's exact Scenario-3 shape: Bank<3>'s layout loop places its
    // resistor array; the board anchors source (0,0) and the whole bank
    // (20,20). Expected: source (0,0), b.rs_0 (20,20), b.rs_1 (25,20),
    // b.rs_2 (30,20) — the complete set, nothing dropped, nothing extra.
    let src = format!(
        "{LIB}
pub subdesign Bank<const N: Int> {{
    ports {{ required IN: Pin }}
    inst rs: [RP; N]
    for w: i in 0..N {{ net _: IN, rs[i].A, rs[i].B }}
    layout {{ for p: i in 0..N {{ place rs[i] at (i * 5mm, 0mm) }} }}
}}
design Board {{
    inst source: SRCP
    subdesign b: Bank<3> {{ IN: source.OUT, }}
    layout {{ place source at (0mm, 0mm)  place b at (20mm, 20mm) }}
}}"
    );
    let mut checked = check(&src);
    let _ = cohdl::pipeline::build_artifacts(&mut checked, &LockState::default()).expect("build");
    let ir = checked.ir.as_ref().unwrap();
    let got = placement_set(
        ir,
        &[
            "Board::source",
            "Board::b::rs_0",
            "Board::b::rs_1",
            "Board::b::rs_2",
        ],
    );
    let want = vec![
        (
            "Board::b::rs_0".to_string(),
            mm(20),
            mm(20),
            0,
            cohdl::ast::PlacementSide::Top,
        ),
        (
            "Board::b::rs_1".to_string(),
            mm(25),
            mm(20),
            0,
            cohdl::ast::PlacementSide::Top,
        ),
        (
            "Board::b::rs_2".to_string(),
            mm(30),
            mm(20),
            0,
            cohdl::ast::PlacementSide::Top,
        ),
        (
            "Board::source".to_string(),
            mm(0),
            mm(0),
            0,
            cohdl::ast::PlacementSide::Top,
        ),
    ];
    assert_eq!(got, want, "the full placement set, exactly");
    // And exactly these four board placements exist — no dropped loop rows,
    // no phantom rows.
    assert_eq!(ir.layout.placements.len(), 4, "complete set, nothing extra");
}

#[test]
fn subdesign_layout_loop_inside_two_instances_composes_independently() {
    // Two use sites of the same looping subdesign: each instance composes
    // through its own anchor, proving the owner is the real node per instance.
    let src = format!(
        "{LIB}
pub subdesign Pair {{
    ports {{ required IN: Pin }}
    inst rs: [RP; 2]
    for w: i in 0..2 {{ net _: IN, rs[i].A, rs[i].B }}
    layout {{ for p: i in 0..2 {{ place rs[i] at (i * 3mm, 1mm) }} }}
}}
design Board {{
    inst source: SRCP
    subdesign a: Pair {{ IN: source.OUT, }}
    subdesign b: Pair {{ IN: source.OUT, }}
    layout {{
        place source at (0mm, 0mm)
        place a at (10mm, 0mm)
        place b at (10mm, 10mm) rotate 90
    }}
}}"
    );
    let mut checked = check(&src);
    let _ = cohdl::pipeline::build_artifacts(&mut checked, &LockState::default()).expect("build");
    let ir = checked.ir.as_ref().unwrap();
    // b rotate 90, top: inv=270 → (dx,dy)→(dy,−dx).
    // rs_0 local (0,1) → (1,0) → (11,10), rotate 0+90=90.
    // rs_1 local (3,1) → (1,−3) → (11,7), rotate 90.
    let got = placement_set(
        ir,
        &[
            "Board::source",
            "Board::a::rs_0",
            "Board::a::rs_1",
            "Board::b::rs_0",
            "Board::b::rs_1",
        ],
    );
    let want = vec![
        (
            "Board::a::rs_0".to_string(),
            mm(10),
            mm(1),
            0,
            cohdl::ast::PlacementSide::Top,
        ),
        (
            "Board::a::rs_1".to_string(),
            mm(13),
            mm(1),
            0,
            cohdl::ast::PlacementSide::Top,
        ),
        (
            "Board::b::rs_0".to_string(),
            mm(11),
            mm(10),
            90,
            cohdl::ast::PlacementSide::Top,
        ),
        (
            "Board::b::rs_1".to_string(),
            mm(11),
            mm(7),
            90,
            cohdl::ast::PlacementSide::Top,
        ),
        (
            "Board::source".to_string(),
            mm(0),
            mm(0),
            0,
            cohdl::ast::PlacementSide::Top,
        ),
    ];
    assert_eq!(got, want, "each instance composes through its own anchor");
}

#[test]
fn subdesign_layout_loop_survives_side_flip_and_outer_override() {
    // A back-side anchor mirrors x before rotating and flips the child side;
    // a board-level reach-in override replaces exactly one loop-authored row.
    let src = format!(
        "{LIB}
pub subdesign Pair {{
    ports {{ required IN: Pin }}
    inst rs: [RP; 2]
    for w: i in 0..2 {{ net _: IN, rs[i].A, rs[i].B }}
    layout {{ for p: i in 0..2 {{ place rs[i] at (i * 4mm, 2mm) }} }}
}}
design Board {{
    inst source: SRCP
    subdesign p: Pair {{ IN: source.OUT, }}
    layout {{
        place source at (0mm, 0mm)
        place p at (10mm, 10mm) rotate 180 side bottom
        place p.rs[1] at (1mm, 1mm) rotate 45
    }}
}}"
    );
    let mut checked = check(&src);
    let _ = cohdl::pipeline::build_artifacts(&mut checked, &LockState::default()).expect("build");
    let ir = checked.ir.as_ref().unwrap();
    // rs_0 local (0,2), bottom: mirror → (0,2); inv(180)=180 → (0,−2);
    // absolute (10,8); rotate = 180+360−0 = 180 (reflection reverses);
    // side flips to bottom.
    let rs0 = ir_place(ir, "Board::p::rs_0");
    assert_eq!((rs0.at.0.femto, rs0.at.1.femto), (mm(10), mm(8)));
    assert_eq!(
        (rs0.rotate, rs0.side),
        (180, cohdl::ast::PlacementSide::Bottom)
    );
    // rs_1 is the explicit override, verbatim.
    let rs1 = ir_place(ir, "Board::p::rs_1");
    assert_eq!(
        (rs1.at.0.femto, rs1.at.1.femto, rs1.rotate, rs1.side),
        (mm(1), mm(1), 45, cohdl::ast::PlacementSide::Top)
    );
}

#[test]
fn subdesign_layout_loop_unanchored_stages_locals_but_not_board() {
    // Without a board anchor the loop-authored defaults stay staged (tooling
    // sees them via local_placements) and the manufacturing layout stays empty.
    let src = format!(
        "{LIB}
pub subdesign Pair {{
    ports {{ required IN: Pin }}
    inst rs: [RP; 2]
    for w: i in 0..2 {{ net _: IN, rs[i].A, rs[i].B }}
    layout {{ for p: i in 0..2 {{ place rs[i] at (i * 3mm, 0mm) }} }}
}}
design Board {{
    inst source: SRCP
    subdesign p: Pair {{ IN: source.OUT, }}
}}"
    );
    let mut checked = check(&src);
    let _ = cohdl::pipeline::build_artifacts(&mut checked, &LockState::default()).expect("build");
    let ir = checked.ir.as_ref().unwrap();
    assert!(
        ir.layout.placements.is_empty(),
        "unanchored: no board placements, got {:?}",
        ir.layout
            .placements
            .iter()
            .map(|p| &p.path)
            .collect::<Vec<_>>()
    );
    let local = &ir.subdesigns["Board::p"].local_placements;
    assert_eq!(local.len(), 2, "staged authored defaults");
    let l0 = local.iter().find(|p| p.path == "Board::p::rs_0").unwrap();
    assert_eq!((l0.at.0.femto, l0.at.1.femto, l0.rotate), (mm(0), mm(0), 0));
    let l1 = local.iter().find(|p| p.path == "Board::p::rs_1").unwrap();
    assert_eq!((l1.at.0.femto, l1.at.1.femto), (mm(3), mm(0)));
}

#[test]
fn subdesign_layout_loop_duplicate_still_e1007() {
    // A loop-authored default for the same target as a sibling default in the
    // SAME subdesign layout is still a duplicate (E1007).
    let src = format!(
        "{LIB}
pub subdesign S {{
    ports {{ required IN: Pin }}
    inst rs: [RP; 2]
    for w: i in 0..2 {{ net _: IN, rs[i].A, rs[i].B }}
    layout {{
        place rs[0] at (0mm, 0mm)
        for p: i in 0..2 {{ place rs[i] at (i * 5mm, 0mm) }}
    }}
}}
design Board {{
    inst source: SRCP
    subdesign s: S {{ IN: source.OUT, }}
    layout {{ place source at (0mm, 0mm)  place s at (1mm, 1mm) }}
}}"
    );
    let files = vec![("src/main.cohdl".to_string(), src)];
    let mut checked = cohdl::pipeline::check_files_in("board", &files, None).expect("selection");
    let _ = cohdl::pipeline::build_artifacts(&mut checked, &LockState::default());
    checked.diags.sort(&checked.sm);
    let r = checked.diags.render(&checked.sm);
    assert!(
        r.contains("E1007") && r.contains("placed more than once"),
        "duplicate within the subdesign's layout (incl. via loop):\n{r}"
    );
}

#[test]
fn nested_subdesign_layout_loops_two_instances_compose() {
    // Nested subdesigns, EACH with a layout for loop, both instantiated at the
    // design level — the inner loop's owner is the inner node, inherited
    // through the middle layer's own loop-bearing layout.
    let src = format!(
        "{LIB}
pub subdesign Leaf {{
    ports {{ required IN: Pin }}
    inst rs: [RP; 2]
    for w: i in 0..2 {{ net _: IN, rs[i].A, rs[i].B }}
    layout {{ for p: i in 0..2 {{ place rs[i] at (i * 2mm, 0mm) }} }}
}}
pub subdesign Mid {{
    ports {{ required IN: Pin }}
    subdesign leaves: [Leaf; 2]
    for w: i in 0..2 {{ net _: IN, leaves[i].IN }}
    layout {{
        for p: i in 0..2 {{ place leaves[i] at (i * 10mm, 0mm) }}
    }}
}}
design Board {{
    inst source: SRCP
    subdesign m1: Mid {{ IN: source.OUT, }}
    subdesign m2: Mid {{ IN: source.OUT, }}
    layout {{
        place source at (0mm, 0mm)
        place m1 at (100mm, 100mm)
        place m2 at (0mm, 200mm)
    }}
}}"
    );
    let mut checked = check(&src);
    let _ = cohdl::pipeline::build_artifacts(&mut checked, &LockState::default()).expect("build");
    let ir = checked.ir.as_ref().unwrap();
    // m1: leaves_0 at (100,100) → rs at (100,100),(102,100);
    //     leaves_1 at (110,100) → rs at (110,100),(112,100).
    // m2: leaves_0 at (0,200) → (0,200),(2,200);
    //     leaves_1 at (10,200) → (10,200),(12,200).
    let got = placement_set(
        ir,
        &[
            "Board::m1::leaves_0::rs_0",
            "Board::m1::leaves_0::rs_1",
            "Board::m1::leaves_1::rs_0",
            "Board::m1::leaves_1::rs_1",
            "Board::m2::leaves_0::rs_0",
            "Board::m2::leaves_0::rs_1",
            "Board::m2::leaves_1::rs_0",
            "Board::m2::leaves_1::rs_1",
        ],
    );
    let mut want = vec![
        ("Board::m1::leaves_0::rs_0", 100, 100),
        ("Board::m1::leaves_0::rs_1", 102, 100),
        ("Board::m1::leaves_1::rs_0", 110, 100),
        ("Board::m1::leaves_1::rs_1", 112, 100),
        ("Board::m2::leaves_0::rs_0", 0, 200),
        ("Board::m2::leaves_0::rs_1", 2, 200),
        ("Board::m2::leaves_1::rs_0", 10, 200),
        ("Board::m2::leaves_1::rs_1", 12, 200),
    ]
    .into_iter()
    .map(|(p, x, y)| {
        (
            p.to_string(),
            mm(x),
            mm(y),
            0,
            cohdl::ast::PlacementSide::Top,
        )
    })
    .collect::<Vec<_>>();
    want.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(got, want, "nested loops, two instances, full set");
}

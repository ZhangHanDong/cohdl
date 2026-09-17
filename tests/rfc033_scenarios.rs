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

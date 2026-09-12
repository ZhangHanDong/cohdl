#!/usr/bin/env python3
"""Run real CLI verdicts, independent topology comparison, and bounded probes."""
import argparse
from collections import Counter
import csv
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile

from verify import EXPORT, HERE, ROOT, child, children, field, sexpr, verify


def replace_once(text, old, new):
    assert text.count(old) == 1, f"mutation anchor changed: {old}"
    return text.replace(old, new)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--record", type=Path, help="write concise machine evidence to this file")
    args = parser.parse_args()
    subprocess.run(["cargo", "build", "--quiet", "--offline", "--locked"], cwd=ROOT, check=True)
    compiler = ROOT / "target/debug/cohdl"
    source = HERE / "sonde.cohdl"
    text = source.read_text()
    results = {
        "compiler_version": subprocess.check_output([str(compiler), "--version"], text=True).strip(),
        "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
        "reference_sha256": hashlib.sha256(EXPORT.read_bytes()).hexdigest(),
        "scope": "full topology check; full production build intentionally blocked by absent parts; separate modern RC build",
        "runs": [],
    }

    def run(name, path, verb="check", expected_exit=0, expected_codes=None, no_std=True):
        command = [str(compiler), verb, str(path)] + (["--no-std"] if no_std else []) + ["--json"]
        process = subprocess.run(command, cwd=ROOT, text=True, capture_output=True)
        assert process.returncode == expected_exit, (name, process.returncode, process.stdout, process.stderr)
        doc = json.loads(process.stdout)
        codes = Counter(d["code"] for d in doc["diagnostics"])
        if expected_codes is not None:
            assert codes == Counter(expected_codes), (name, codes, doc)
        # Keep every code count and one complete representative diagnostic per code.
        examples = {}
        for d in doc["diagnostics"]:
            examples.setdefault(d["code"], d["message"])
        results["runs"].append({
            "name": name, "command": [str(x).replace(str(ROOT) + "/", "").replace(str(path), "<probe>" if path.parent.name.startswith("sonde-probes-") else str(path)) for x in command],
            "exit": process.returncode, "verdict": doc["verdict"],
            "diagnostic_counts": dict(sorted(codes.items())), "diagnostic_examples": examples,
        })
        return doc

    for path in (source, HERE / "rc-control"):
        subprocess.run([str(compiler), "fmt", str(path), "--check"], cwd=ROOT, check=True)
    run("full_check", source, expected_codes={})
    results["topology"] = verify(source)
    run("full_build_missing_procurement", source, "build", 1, {"E801": 25})

    with tempfile.TemporaryDirectory(prefix="sonde-probes-") as temp:
        temp = Path(temp)
        def probe(name, mutated, expected_exit=0, expected_codes=None):
            path = temp / f"{name}.cohdl"
            path.write_text(mutated)
            run(name, path, expected_exit=expected_exit, expected_codes=expected_codes)
            return path

        probe("wrong_resistance_unit", replace_once(text, "AxialResistor<5.1kohm>", "AxialResistor<5.1nF>"), 1, {"E112": 1})
        probe("missing_nc_endpoint", replace_once(text, "nc: J1.P1,", "nc:"), 1, {"E701": 1})
        probe("connected_and_nc", text.rsplit("}", 1)[0] + "    net CONFLICT: U2.O2, J1.P8\n}\n", 1, {"E702": 1})
        probe("two_output_drivers", text.rsplit("}", 1)[0] + "    net SHORT: U1.O2, U1.O3\n}\n", 1, {"D004": 1})

        swapped = replace_once(text, "buffered_path(J1.P2, U1.D2", "buffered_path(J1.P3, U1.D2")
        swapped = replace_once(swapped, "buffered_path(J1.P3, U1.D3", "buffered_path(J1.P2, U1.D3")
        path = probe("swapped_host_endpoints", swapped, expected_codes={})
        try:
            verify(path)
        except AssertionError as error:
            assert "connected endpoint partitions differ" in str(error), str(error)
            results["independent_verifier_mutation"] = {"compiler_exit": 0, "rejected": True, "reason": str(error)}
        else:
            raise AssertionError("independent verifier missed swapped endpoints")

        # Synthetic limits: deliberately NOT asserted ratings of the real 74LS125.
        # The arbitrary lower-bound spec is accepted metadata, not a checked bound.
        limits = replace_once(text, "device Quad74LS125 {", "device Quad74LS125 {\n    spec { voltage_rating: 5V, supply_min: 4V }")
        low = replace_once(limits, "net VCC:", "net VCC [1V]:")
        probe("synthetic_minimum_voltage_gap", low, expected_codes={})
        high = replace_once(limits, "net VCC:", "net VCC [6V]:")
        probe("synthetic_maximum_voltage_detected", high, 1, {"D001": 2})
        native = replace_once(text, "required O1: 3 [output]", "required O1: 3 [tri_state]")
        probe("native_tristate_role_gap", native, 1, {"E010": 1})

    run("rc_control_build", HERE / "rc-control", "build", expected_codes={}, no_std=False)
    out = HERE / "rc-control/out"
    artifacts = {p.relative_to(out).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(out.rglob("*")) if p.is_file()}
    run("rc_control_repeat_build", HERE / "rc-control", "build", expected_codes={}, no_std=False)
    assert artifacts == {p.relative_to(out).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(out.rglob("*")) if p.is_file()}, "repeat build changed bytes"
    with (out / "sonde-rc-control-bom.csv").open(newline="") as f:
        bom = list(csv.DictReader(f))
    assert {(r["Designator"], r["Comment"]) for r in bom} == {("C1", "CC0603JRNPO9BN101"), ("R1", "RC0603FR-07100RL")}
    netlist = sexpr((out / "sonde-rc-control.net").read_text())
    rc_nets = {field(n, "name"): {(field(p, "ref"), field(p, "pin")) for p in children(n, "node")} for n in children(child(netlist, "nets"), "net")}
    assert rc_nets == {"INPUT": {("R1", "1")}, "RC_NODE": {("R1", "2"), ("C1", "1")}, "GND": {("C1", "2")}}, "real emitted RC netlist endpoints differ"
    assert {field(c, "ref"): field(c, "value") for c in children(child(netlist, "components"), "comp")} == {"R1": "100ohm", "C1": "100pF"}
    results["rc_control"] = {"bom": bom, "artifact_sha256": artifacts, "repeat_bytes_identical": True, "emitted_netlist_endpoint_check": True}
    results["format_checks"] = "full model and RC project passed"
    output = json.dumps(results, indent=2, ensure_ascii=False) + "\n"
    if args.record:
        args.record.write_text(output)
    print(output)


if __name__ == "__main__":
    main()

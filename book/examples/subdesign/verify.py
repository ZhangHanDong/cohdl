#!/usr/bin/env python3
"""Run the RFC-032 lesson with an explicitly selected main-branch compiler."""
import argparse
import csv
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("sonde_net_reader", HERE.parent / "sonde/verify.py")
net_reader = importlib.util.module_from_spec(spec)
spec.loader.exec_module(net_reader)


def run(compiler, verb, project, expected=0):
    cmd = [str(compiler), verb, str(project), "--json"]
    p = subprocess.run(cmd, text=True, capture_output=True)
    data = json.loads(p.stdout)
    assert p.returncode == expected, (cmd, p.returncode, p.stdout, p.stderr)
    return {"command": cmd, "exit_code": p.returncode, "result": data}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--compiler", type=Path, required=True)
    ap.add_argument("--record", type=Path)
    args = ap.parse_args()
    compiler = args.compiler.resolve()
    source = (HERE / "src/main.cohdl").read_text()
    result = {
        "compiler": str(compiler),
        "compiler_sha256": hashlib.sha256(compiler.read_bytes()).hexdigest(),
        "source_sha256": hashlib.sha256(source.encode()).hexdigest(),
        "check": run(compiler, "check", HERE),
        "build": run(compiler, "build", HERE),
    }
    artifacts = result["build"]["result"]["build"]
    paths = [Path(artifacts[k]) for k in ("netlist", "bom", "layout")]
    paths += [Path(p) for p in artifacts["kicad_mod"]]
    paths += [HERE / "design.lock", HERE / "cohdl.lock"]
    before = {str(p.relative_to(HERE)): p.read_bytes() for p in paths}
    result["repeat_build"] = run(compiler, "build", HERE)
    assert all(p.read_bytes() == before[str(p.relative_to(HERE))] for p in paths)
    result["artifact_sha256"] = {
        name: hashlib.sha256(data).hexdigest() for name, data in before.items()
    }
    tree = net_reader.sexpr(Path(artifacts["netlist"]).read_text())
    components = net_reader.children(net_reader.child(tree, "components"), "comp")
    refs = sorted(net_reader.field(c, "ref") for c in components)
    assert refs == ["C1", "C2", "J1", "R1", "R2"], refs
    nets = {
        net_reader.field(n, "name"): sorted(
            (net_reader.field(p, "ref"), net_reader.field(p, "pin"))
            for p in net_reader.children(n, "node")
        )
        for n in net_reader.children(net_reader.child(tree, "nets"), "net")
    }
    assert nets == {
        "GND": [("C1", "2"), ("C2", "2"), ("J1", "5")],
        "IN0": [("J1", "1"), ("R1", "1")],
        "IN1": [("J1", "3"), ("R2", "1")],
        "OUT0": [("C1", "1"), ("J1", "2"), ("R1", "2")],
        "OUT1": [("C2", "1"), ("J1", "4"), ("R2", "2")],
    }, nets
    placements = {
        p["instance"]: p["at"]
        for p in json.loads(Path(artifacts["layout"]).read_text())["placements"]
    }
    assert placements == {
        "RcPair::connector": [10, 6],
        "RcPair::channels_0::r": [10, 15],
        "RcPair::channels_0::c": [13, 15],
        "RcPair::channels_1::r": [20, 15],
        "RcPair::channels_1::c": [24, 17],
    }, placements
    bom = list(csv.DictReader(io.StringIO(Path(artifacts["bom"]).read_text())))
    assert sorted(ref for row in bom for ref in row["Designator"].split(",")) == refs
    assert len(bom) == 3, bom
    assert {tuple(sorted(row["Designator"].split(","))) for row in bom} == {
        ("C1", "C2"), ("R1", "R2"), ("J1",),
    }, bom
    result["verified"] = {
        "components": refs, "nets": nets, "placements_mm": placements,
        "bom_rows": len(bom), "repeated_artifact_bytes_identical": True,
    }
    probes = [
        ("missing_required_port", source.replace("connector.P2, channels[0].OUT", "connector.P2"), "E1302"),
        ("electrical_reach_in", source.replace("connector.P2, channels[0].OUT", "connector.P2, channels[0].r.B"), "E010"),
        ("internal_name_is_not_a_port", source.replace("net IN0: connector.P1, channels[0].IN", "net IN0: connector.P1, channels[0].IN, channels[0].r"), "E1301"),
        ("wrong_port_type", source.replace("required IN: Pin", "required IN: Voltage"), "E1303"),
        ("nc_port", source.replace("nc: connector.P6", "nc: connector.P6, channels[0].OUT"), "E1306"),
        ("cross_package_fn", source.replace("    nc: connector.P6", "    passive::bulk_10u(connector.P1, connector.P5)\n    nc: connector.P6"), None),
    ]
    result["probes"] = {}
    for name, mutated, error in probes:
        with tempfile.TemporaryDirectory(prefix="cohdl-subdesign-probe-") as tmp:
            project = Path(tmp)
            (project / "src").mkdir()
            for name_in in ("cohdl.toml", "cohdl.lock"):
                shutil.copyfile(HERE / name_in, project / name_in)
            (project / "src/main.cohdl").write_text(mutated)
            probe = run(compiler, "check", project, expected=1 if error else 0)
            codes = sorted({d["code"] for d in probe["result"]["diagnostics"]})
            assert error in codes if error else not codes, (name, codes)
            probe["observed_codes"] = codes
            result["probes"][name] = probe
    text = json.dumps(result, ensure_ascii=False, indent=2) + "\n"
    if args.record:
        args.record.write_text(text)
    print(json.dumps({"verified": result["verified"], "probes": {
        k: v["observed_codes"] for k, v in result["probes"].items()
    }}, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()

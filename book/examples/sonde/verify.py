#!/usr/bin/env python3
"""Independent exported-netlist vs checked-IR topology check (Python stdlib only)."""
import argparse
from collections import Counter
from decimal import Decimal
import json
from pathlib import Path
import re
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
EXPORT = ROOT / "book/src/learning/evidence/2026-09-08-sonde-full/sonde.net"


def sexpr(text):
    tokens = re.findall(r'"(?:\\.|[^"\\])*"|[()]|[^\s()]+', text)
    stack, result = [], None
    for token in tokens:
        if token == "(":
            child = []
            if stack:
                stack[-1].append(child)
            stack.append(child)
        elif token == ")":
            result = stack.pop()
        else:
            stack[-1].append(json.loads(token) if token.startswith('"') else token)
    assert not stack and result[0] == "export", "malformed exported netlist"
    return result


def children(node, tag):
    return [x for x in node[1:] if isinstance(x, list) and x[0] == tag]


def child(node, tag):
    found = children(node, tag)
    assert len(found) == 1, (tag, found)
    return found[0]


def field(node, tag):
    return child(node, tag)[1]


def reference(path=EXPORT):
    tree = sexpr(path.read_text())
    components = {}
    for c in children(child(tree, "components"), "comp"):
        ref = field(c, "ref")
        assert ref not in components
        components[ref] = {"value": field(c, "value"), "footprint": field(c, "footprint")}
    nets, nc = [], set()
    for n in children(child(tree, "nets"), "net"):
        nodes = children(n, "node")
        endpoints = frozenset((field(p, "ref"), field(p, "pin")) for p in nodes)
        if all("no_connect" in field(p, "pintype") for p in nodes):
            assert len(nodes) == 1
            nc.update(endpoints)
        else:
            assert not any("no_connect" in field(p, "pintype") for p in nodes)
            nets.append((field(n, "name"), endpoints))
    return components, nets, nc


def inspect(source):
    run = subprocess.run([
        "cargo", "run", "--quiet", "--offline", "--locked", "--manifest-path",
        str(HERE / "inspect/Cargo.toml"), "--", str(source),
    ], cwd=ROOT, text=True, capture_output=True)
    if run.returncode:
        raise AssertionError(f"checked-IR inspection failed ({run.returncode}): {run.stderr}")
    instances, pins, nets, nc = {}, {}, {}, set()
    for line in run.stdout.splitlines():
        row = line.split("\t")
        if row[0] == "I":
            assert row[1] not in instances
            instances[row[1]] = {"device": row[2], "specs": {}}
        elif row[0] == "S":
            instances[row[1]]["specs"][row[2]] = (row[3], int(row[4]))
        elif row[0] == "P":
            pins[(row[1], row[2])] = row[3]
        elif row[0] == "N":
            nets.setdefault(row[1], set()).add((row[2], row[3]))
        elif row[0] == "X":
            nc.add((row[1], row[2]))
        else:
            raise AssertionError(f"unknown record: {row}")
    return instances, pins, list(map(frozenset, nets.values())), nc


def verify(source):
    expected, named_nets, expected_nc = reference()
    instances, pins, nets, nc = inspect(source)
    assert set(instances) == set(expected), "component references differ"
    expected_nets = [n for _, n in named_nets]
    assert Counter(nets) == Counter(expected_nets), "connected endpoint partitions differ"
    assert nc == expected_nc, "explicit NC endpoints differ"
    expected_endpoints = set().union(*expected_nets, expected_nc)
    assert set(pins) == expected_endpoints, "physical pin universe differs"
    tree = sexpr(EXPORT.read_text())
    for net in children(child(tree, "nets"), "net"):
        for node in children(net, "node"):
            role = field(node, "pintype").split("+")[0]
            if role == "tri_state":
                role = "output"  # explicitly documented conservative abstraction
            assert pins[(field(node, "ref"), field(node, "pin"))] == role, "pin role differs"
    # Cross-check a second export: live PCB pads from Konnect, no source parsing.
    pad_rows = json.loads(EXPORT.with_name("konnect.json").read_text())
    pads, pcb_refs = {}, set()
    for row in pad_rows:
        if row["tool"] != "get_component_pads":
            continue
        ref = row["arguments"]["reference"]
        assert ref not in pcb_refs
        pcb_refs.add(ref)
        data = json.loads(row["result"]["content"][0]["text"])
        for p in data["pads"]:
            key = ref, p["number"]
            assert key not in pads
            pads[key] = p["net"]
    assert pcb_refs == set(expected)
    assert set(pads) == expected_endpoints, "PCB pad universe differs"
    for net in children(child(tree, "nets"), "net"):
        for node in children(net, "node"):
            key = field(node, "ref"), field(node, "pin")
            assert pads[key] == field(net, "name"), "schematic/PCB net assignment differs"
    for ref, comp in expected.items():
        actual = instances[ref]
        if ref.startswith("R"):
            assert actual["device"] == "AxialResistor", (ref, "device identity")
            field_name, unit = "resistance", "Resistance"
            value = comp["value"].replace("K", "k").replace(",", ".")
            factor = 1000 if value.endswith("k") else 1
            femto = int(Decimal(value.rstrip("k")) * factor * 10**15)
        elif ref.startswith("C"):
            assert actual["device"] == ("PolarizedCapacitor" if ref == "C1" else "DiscCapacitor"), (ref, "device identity")
            field_name, unit = "capacitance", "Capacitance"
            value = comp["value"]
            factor = {"uF": 10**9, "pF": 1000}[value[-2:]]
            femto = int(Decimal(value[:-2]) * factor)
        else:
            device = {"BAT46": "BAT46", "74LS125": "Quad74LS125", "DB25MALE": "DB25", "DB9MALE": "DB9", "CONN_6": "Header6"}[comp["value"]]
            assert actual["device"] == device, (ref, "device identity")
            continue
        assert actual["specs"][field_name] == (unit, femto), (ref, "component value")
    return {"instances": len(instances), "connected_net_partitions": len(nets), "connected_endpoints": sum(map(len, nets)), "nc_endpoints": len(nc), "physical_pins": len(pins), "konnect_pads": len(pads)}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", nargs="?", type=Path, default=HERE / "sonde.cohdl")
    args = parser.parse_args()
    print(json.dumps(verify(args.source), indent=2))

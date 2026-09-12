#!/usr/bin/env python3
"""Verify a literal RFC-032 baseline; never evaluates proposed M2 syntax."""
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
import tomllib

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--compiler', type=Path, required=True)
    parser.add_argument('--compiler-source', type=Path,
                        help='Checkout used to build --compiler; required when recording evidence')
    parser.add_argument('--record', type=Path)
    args = parser.parse_args()
    if args.record and not args.compiler_source:
        parser.error('--record requires --compiler-source (build that checkout first)')
    compiler = args.compiler.resolve()
    loader = importlib.util.spec_from_file_location('net_reader', ROOT / 'book/examples/sonde/verify.py')
    reader = importlib.util.module_from_spec(loader)
    loader.loader.exec_module(reader)
    record = {'compiler': str(compiler), 'compiler_sha256': hashlib.sha256(compiler.read_bytes()).hexdigest(),
              'scope': 'Literal current syntax, real library parts, no M2 evaluation or physical clearance checks', 'stages': {}}
    record['compiler_version'] = subprocess.check_output([str(compiler), '--version'], text=True).strip()
    if args.compiler_source:
        checkout = args.compiler_source.resolve()
        def git(*git_args):
            return subprocess.check_output(['git', '-C', str(checkout), *git_args], text=True).strip()
        record['compiler_source'] = {
            'checkout': str(checkout), 'commit': git('rev-parse', 'HEAD'),
            'tracked_changes': git('status', '--porcelain', '--untracked-files=no'),
            'binding': 'Caller declares this checkout produced the executable; build it before this command.',
        }
    record['verifier_sha256'] = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    with tempfile.TemporaryDirectory(prefix='cohdl-rc-workflow-') as tmp:
        project = Path(tmp)
        (project / 'src').mkdir()
        for filename in ('cohdl.toml', 'cohdl.lock'):
            shutil.copyfile(HERE / filename, project / filename)
        record['input_sha256'] = {f: hashlib.sha256((HERE/f).read_bytes()).hexdigest() for f in ('cohdl.toml','cohdl.lock')}
        def build():
            cmd = [str(compiler), 'build', str(project), '--json']
            out = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True)
            result = json.loads(out.stdout)
            assert out.returncode == 0, (out.returncode, result, out.stderr)
            assert hashlib.sha256((project/'cohdl.lock').read_bytes()).hexdigest() == record['input_sha256']['cohdl.lock'], 'dependency lock changed during build'
            return result
        def artifacts(result):
            out = result['build']
            paths = [Path(out[k]) for k in ('netlist','bom','layout')]
            paths += [Path(f) for f in out['kicad_mod']]
            paths += [project/'design.lock', project/'cohdl.lock']
            return {str(f.relative_to(project)): hashlib.sha256(f.read_bytes()).hexdigest() for f in paths}
        def net_partitions(output):
            lock = tomllib.loads((project/'design.lock').read_text())['designators']
            inverse = {v:k for k,v in lock.items()}
            tree = reader.sexpr(Path(output['netlist']).read_text())
            return {frozenset((inverse[reader.field(m,'ref')],reader.field(m,'pin'))
                             for m in reader.children(net,'node'))
                    for net in reader.children(reader.child(tree,'nets'),'net')}
        previous = None
        for filename, n, override in [('current-10.cohdl',10,False),('current-10-override.cohdl',10,True),('current-12.cohdl',12,True)]:
            shutil.copyfile(HERE/filename, project/'src/main.cohdl')
            result = build()
            hashes = artifacts(result)
            assert hashes == artifacts(build()), 'repeat-build bytes changed'
            output = result['build']
            designators = tomllib.loads((project/'design.lock').read_text())['designators']
            assert len(designators) == 3*n
            inverse = {v:k for k,v in designators.items()}
            tree = reader.sexpr(Path(output['netlist']).read_text())
            comps = reader.children(reader.child(tree,'components'),'comp')
            assert len(comps) == 3*n
            nets = set()
            for net in reader.children(reader.child(tree,'nets'),'net'):
                nets.add(frozenset((inverse[reader.field(m,'ref')],reader.field(m,'pin')) for m in reader.children(net,'node')))
            expected = set()
            for i in range(n):
                prefix=f'RcWorkflow::channels_{i}'
                conn=f'RcWorkflow::connectors_{i}'
                expected.add(frozenset([(conn,'1'),(prefix+'::r','1')]))
                expected.add(frozenset([(conn,'2'),(prefix+'::r','2'),(prefix+'::c','1')]))
            expected.add(frozenset((f'RcWorkflow::{kind}_{i}{tail}',pin) for i in range(n) for kind,tail,pin in [('connectors','','5'),('channels','::c','2')]))
            assert nets == expected, 'unexpected physical endpoint partitions'
            assert sum(map(len,nets)) == 7*n
            places={v['instance']:v['at'] for v in json.loads(Path(output['layout']).read_text())['placements']}
            expected_places={}
            for i in range(n):
                expected_places[f'RcWorkflow::channels_{i}::r']=[10+8*i,15]
                expected_places[f'RcWorkflow::channels_{i}::c']=[13+8*i,15]
                expected_places[f'RcWorkflow::connectors_{i}']=[10+8*i,5]
            if override: expected_places['RcWorkflow::channels_6::c']=[62,17]
            assert places == expected_places
            bom=list(csv.DictReader(io.StringIO(Path(output['bom']).read_text())))
            assert sorted(ref for row in bom for ref in row['Designator'].split(',')) == sorted(inverse)
            assert sorted(len(row['Designator'].split(',')) for row in bom) == [n,n,n]
            # Expected primary parts from the locked passive/connectors declarations.
            expected_bom = {
                ('Yageo', 'CC0603KRX7R6BB104', 'CHIP_0603'):
                    {designators[f'RcWorkflow::channels_{i}::c'] for i in range(n)},
                ('Yageo', 'RC0603FR-071KL', 'CHIP_0603'):
                    {designators[f'RcWorkflow::channels_{i}::r'] for i in range(n)},
                ('Samtec', 'SSW-103-22-SM-D-VS', 'FP_Socket_2x3_254_SMD'):
                    {designators[f'RcWorkflow::connectors_{i}'] for i in range(n)},
            }
            assert {(row['Manufacturer'], row['Comment'], row['Footprint']):
                    set(row['Designator'].split(',')) for row in bom} == expected_bom, 'BOM parts or group membership changed'
            # Carry the same design.lock between edits. Restrict new connectivity to old endpoints.
            if previous:
                old_lock,old_nets,old_places=previous
                assert all(designators[k] == v for k,v in old_lock.items())
                restricted={frozenset((path,pin) for path,pin in net if path in old_lock) for net in nets}
                restricted.discard(frozenset())
                assert restricted == old_nets
                changed={k for k in old_places if old_places[k] != places[k]}
                assert changed == ({'RcWorkflow::channels_6::c'} if n == 10 else set())
            record['stages'][filename]={'source_sha256':hashlib.sha256((HERE/filename).read_bytes()).hexdigest(),
                'build_exit_code':0,'diagnostics':result['diagnostics'],'physical_components':3*n,
                'passives':2*n,'net_partitions':len(nets),'connected_pad_endpoints':sum(map(len,nets)),
                'placements_mm':places,'designators':designators,'artifact_sha256':hashes,
                'bom_primary_parts_verified':True, 'dependency_lock_unchanged':True,
                'repeat_build_bytes_equal':True,'surviving_designators_and_restricted_connectivity_preserved':bool(previous)}
            previous=designators,nets,places
        # Each probe starts from the same ten-channel source, never another probe.
        baseline = (HERE/'current-10.cohdl').read_text()
        probes = [
            ('wrong_position_unit', '(10mm, 15mm)', '(10V, 15mm)', ['E1007']),
            ('missing_capacitor_ground', 'net _: GND, c.B', 'net _: GND', ['E701']),
            ('missing_external_output', 'net OUT0: connectors[0].P2, channels[0].OUT', '', ['E1302', 'E701']),
            ('array_index_out_of_bounds', 'place channels[0] at', 'place channels[10] at', ['E202']),
            ('short_two_outputs', 'net IN0:', 'net SHORT: channels[0].OUT, channels[1].OUT\n    net IN0:', []),
        ]
        record['probes'] = {}
        (project/'src/main.cohdl').write_text(baseline)
        baseline_nets = net_partitions(build()['build'])
        for name, old, new, expected_codes in probes:
            assert old in baseline
            source = baseline.replace(old, new, 1)
            (project/'src/main.cohdl').write_text(source)
            out = subprocess.run([str(compiler), 'check', str(project), '--json'],
                                 cwd=ROOT, capture_output=True, text=True)
            result = json.loads(out.stdout)
            codes = sorted({d['code'] for d in result['diagnostics']})
            assert hashlib.sha256((project/'cohdl.lock').read_bytes()).hexdigest() == record['input_sha256']['cohdl.lock'], 'dependency lock changed during check'
            assert codes == expected_codes, (name, codes, expected_codes)
            if expected_codes:
                assert out.returncode == 1, (name, out.returncode, result)
            else:
                assert out.returncode == 0, (name, result)
                assert not result['diagnostics'], (name, result)
                actual = net_partitions(build()['build'])
                a = next(net for net in baseline_nets if ('RcWorkflow::connectors_0', '2') in net)
                b = next(net for net in baseline_nets if ('RcWorkflow::connectors_1', '2') in net)
                assert actual == (baseline_nets - {a, b}) | {a | b}
            record['probes'][name] = {
                'source_sha256': hashlib.sha256(source.encode()).hexdigest(),
                'check_exit_code': out.returncode, 'codes': codes, 'diagnostics': result['diagnostics'],
            }
            if not expected_codes:
                record['probes'][name].update(net_partitions=len(actual),
                    independent_channel_contract_passed=False, exact_output_merge_verified=True)
    if args.record: args.record.write_text(json.dumps(record,indent=2)+'\n')
    print(json.dumps({f:{k:v for k,v in stage.items() if k in ('physical_components','passives','net_partitions','build_exit_code','repeat_build_bytes_equal')} for f,stage in record['stages'].items()},indent=2))
    print(json.dumps({name: {k:v for k,v in probe.items() if k in ('check_exit_code','codes','net_partitions')}
                      for name,probe in record['probes'].items()}, indent=2))

if __name__ == '__main__': main()

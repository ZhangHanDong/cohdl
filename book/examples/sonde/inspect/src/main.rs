//! Check-only topology observer. No part binding, allocator, or emitter calls.
use cohdl::pipeline::check_files_in_with_deps;

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: inspector SOURCE.cohdl");
    let source = std::fs::read_to_string(path).expect("read source");
    let checked =
        check_files_in_with_deps("sonde", &[], &[("sonde.cohdl".to_string(), source)], None)
            .expect("run check pipeline");
    if checked.diags.has_errors() || checked.selection_error.is_some() {
        for d in checked.diags.iter() {
            eprintln!("{}: {}", d.code, d.message);
        }
        eprintln!("selection: {:?}", checked.selection_error);
        std::process::exit(1);
    }
    let ir = checked.ir.as_ref().expect("checked design IR");
    let reference = |path: &String| {
        ir.instances[path]
            .designator_override
            .as_ref()
            .expect("all teaching instances require explicit reference")
            .0
            .as_str()
    };
    let physical = |path: &String, logical: &String| {
        let inst = &ir.instances[path];
        checked.world.devices[&inst.device]
            .pins_for(inst.variant.as_deref())
            .iter()
            .find(|p| p.name.name == *logical)
            .expect("resolved logical pin")
    };
    for (path, inst) in &ir.instances {
        let r = reference(path);
        println!("I\t{r}\t{}", cohdl::resolve::short(&inst.device));
        for (key, value) in &inst.specs {
            println!("S\t{r}\t{key}\t{}\t{}", value.unit.type_name(), value.femto);
        }
        for pin in checked.world.devices[&inst.device].pins_for(inst.variant.as_deref()) {
            for n in &pin.numbers {
                println!("P\t{r}\t{}\t{}", n.text, pin.role_or_default().name());
            }
        }
    }
    for (index, net) in ir.nets.iter().enumerate() {
        for (path, logical) in &net.members {
            for n in &physical(path, logical).numbers {
                println!("N\t{index}\t{}\t{}", reference(path), n.text);
            }
        }
    }
    for (path, logical) in &ir.nc_pins {
        for n in &physical(path, logical).numbers {
            println!("X\t{}\t{}", reference(path), n.text);
        }
    }
}

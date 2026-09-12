use cohdl::pipeline::check_files_in_with_deps;
fn main() {
    let file = std::env::args().nth(1).expect("source path");
    let source = std::fs::read_to_string(file).unwrap();
    let checked = check_files_in_with_deps("fixture", &[], &[("fixture.cohdl".into(), source)], None).unwrap();
    assert!(!checked.diags.has_errors() && checked.selection_error.is_none());
    let ir = checked.ir.as_ref().unwrap();
    println!("instances\t{}", ir.instances.len());
    println!("nets\t{}", ir.nets.len());
    println!("connected_pins\t{}", ir.nets.iter().map(|n| n.members.len()).sum::<usize>());
    println!("nc\t{}", ir.nc_pins.len());
    for (path, inst) in &ir.instances { println!("instance\t{}\t{}", path, cohdl::resolve::short(&inst.device)); }
    for net in &ir.nets {
        let mut members: Vec<_> = net.members.iter().map(|(p, pin)| format!("{}.{}", p, pin)).collect();
        members.sort();
        println!("net\t{}", members.join("\t"));
    }
}

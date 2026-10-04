// Compiles the C solver into a static library that is linked into whichever
// binary depends on this crate. `csrc/main.c` is the standalone host harness
// (it defines `main`) and is deliberately left out; `just solver` builds that.
fn main() {
    println!("cargo:rerun-if-changed=csrc/solver.c");
    println!("cargo:rerun-if-changed=csrc/solver.h");
    cc::Build::new()
        .file("csrc/solver.c")
        .include("csrc")
        .compile("mr_solver");
}

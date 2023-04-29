fn main() {
    cc::Build::new().file("evm.c").shared_flag(true).compile("evm");
    cc::Build::new().file("sol.c").shared_flag(true).compile("sol");
}

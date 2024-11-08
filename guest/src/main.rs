extern crate powdr_riscv_runtime;

use powdr_riscv_runtime::io::read;

pub fn main() {
    let a: u8 = read(1);
    let b: u8 = read(2);
    let c: u8 = read(3);
    assert_eq!(a + b, c);
}

extern crate powdr_riscv_runtime;
use powdr_number::GoldilocksField;

use powdr_riscv_runtime::io::read;

use powdr_plonky3::{verify2, FieldElementMap};

static PROOF_CONTENT: &'static [u8] = include_bytes!("../../powdr-target/chunk_0/guest_proof.bin");
static VKEY_CONTENT: &'static [u8] = include_bytes!("../../powdr-target/vkey.bin");
static SPLIT_CONTENT: &'static [u8] = include_bytes!("../../powdr-target/split.bin");

pub fn main() {
    //let proof = read(1);
    let proof = bincode::deserialize(PROOF_CONTENT)
        .map_err(|e| format!("Failed to deserialize proof: {e}"))
        .unwrap();
    //let verifying_key = read(2);
    let verifying_key = bincode::deserialize(VKEY_CONTENT)
        .map_err(|e| format!("Failed to deserialize vkey: {e}"))
        .unwrap();
    //let split = read(3);
    let split = bincode::deserialize(SPLIT_CONTENT)
        .map_err(|e| format!("Failed to deserialize split: {e}"))
        .unwrap();
    let mut challenger = GoldilocksField::get_challenger();
    let public_inputs = read(4);

    let _ = verify2::<GoldilocksField>(
        Some(&verifying_key),
        &split,
        &mut challenger,
        &proof,
        &public_inputs,
    );
}

use powdr::GoldilocksField;
use powdr::Session;
use powdr_plonky3::{verify2, FieldElementMap, Proof};

fn main() {
    env_logger::init();

    // First run basic guest.
    basic_guest();

    // When you have the basic guest artifacts, run recursion.
    //recursion();
}

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Read};

fn read_binary_file(file_path: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn recursion() {
    let session = Session::builder()
        .guest_path("./guest-rec")
        .out_path("powdr-target-rec")
        .build();

    let proof = read_binary_file("powdr-target/chunk_0/guest_proof.bin").unwrap();
    let vkey = read_binary_file("powdr-target/vkey.bin").unwrap();
    let split = read_binary_file("powdr-target/split.bin").unwrap();

    let proof: Proof<_> = bincode::deserialize(&proof)
        .map_err(|e| format!("Failed to deserialize proof: {e}"))
        .unwrap();
    let vkey = bincode::deserialize(&vkey)
        .map_err(|e| format!("Failed to deserialize vkey: {e}"))
        .unwrap();
    let split: BTreeMap<_, _> = bincode::deserialize(&split)
        .map_err(|e| format!("Failed to deserialize split: {e}"))
        .unwrap();

    let mut challenger = GoldilocksField::get_challenger();

    let mut publics: BTreeMap<_, _> = Default::default();
    split.keys().for_each(|k: &String| {
        publics.insert(k.clone(), vec![vec![]]);
    });

    // This runs the native version just to check.
    let verified =
        verify2::<GoldilocksField>(Some(&vkey), &split, &mut challenger, &proof, &publics);
    assert!(verified.is_ok());

    let mut session = session
        .write(1, &proof)
        .write(2, &vkey)
        .write(3, &split)
        .write(4, &publics);

    // Dry run to test execution.
    session.run();

    // Compute the proof.
    //session.prove();
}

fn basic_guest() {
    let mut session = Session::builder()
        .guest_path("./guest")
        .out_path("powdr-target")
        .build()
        .write(1, &1u8)
        .write(2, &2u8)
        .write(3, &3u8);

    // Dry run to test execution.
    session.run();

    // Compute the proof.
    session.prove();
}

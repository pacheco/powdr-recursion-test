use powdr::number::LargeInt;
use powdr::riscv::RuntimeLibs;
use powdr::FieldElement;
use powdr::GoldilocksField;
use powdr::Session;
use powdr_plonky3::ConstraintSystem;
use powdr_plonky3::{verify, FieldElementMap, Proof};
use powdr_riscv_executor::poseidon_gl::poseidon_gl_inplace;

fn main() {
    env_logger::init();

    // First run basic guest.
    basic_guest();

    println!("GUEST DONE! ---------------------------------");

    // When you have the basic guest artifacts, run recursion.
    recursion();
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

fn publics_from_commits<I: Iterator<Item = u64>>(commits: I) -> [u32; 8] {
    let mut state = [0.into(); 12];
    let mut used = 0;
    for e in commits.chain(std::iter::once(1u64.into())) {
        state[used + 4] = e.into();
        used += 1;
        if used == 4 {
            used = 0;
            poseidon_gl_inplace::<GoldilocksField>(&mut state);
        }
    }
    if used != 0 {
        for n in state[used + 4..8].iter_mut() {
            *n = 0.into();
        }
        poseidon_gl_inplace::<GoldilocksField>(&mut state);
    }
    let out = state[0..4]
        .into_iter()
        .flat_map(|e| {
            let i = e.to_integer().try_into_u64().unwrap();
            [i as u32, (i >> 32) as u32]
        })
        .collect::<Vec<_>>();
    out.try_into().unwrap()
}

fn recursion() {
    let session = Session::builder()
        .guest_path("./guest-rec")
        .out_path("powdr-target-rec")
        .chunk_size_log2(18)
        .precompiles(RuntimeLibs::new().with_poseidon2())
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
    let split: BTreeMap<String, ConstraintSystem<GoldilocksField>> = bincode::deserialize(&split)
        .map_err(|e| format!("Failed to deserialize split: {e}"))
        .unwrap();

    let mut challenger = GoldilocksField::get_challenger();

    let public_values: Vec<GoldilocksField> = publics_from_commits(std::iter::empty())
        .into_iter()
        .map(|v| v.into())
        .collect();
    println!("publics: {public_values:?}");

    let mut publics: BTreeMap<_, _> = Default::default();
    split.keys().for_each(|k| {
        if k == "main_publics" {
            publics.insert(k.clone(), vec![public_values.clone()]);
        } else {
            publics.insert(k.clone(), vec![vec![]]);
        }
    });

    println!("NATIVE CHECK ---------------------------------");

    let mut session = session
        .write(1, &proof)
        .write(2, &vkey)
        .write(3, &split)
        .write(4, &publics);

    // This runs the native version just to check.
    let verified = verify::<GoldilocksField>(Some(&vkey), &split, &mut challenger, &proof, publics);
    assert!(verified.is_ok());

    println!("NATIVE OK! ---------------------------------");

    // Dry run to test execution.
    session.run();

    // Compute the proof.
    //session.prove();
}

fn basic_guest() {
    let mut session = Session::builder()
        .guest_path("./guest")
        .out_path("powdr-target")
        .chunk_size_log2(18)
        .build()
        .write(1, &1u8)
        .write(2, &2u8)
        .write(3, &3u8);

    // Dry run to test execution.
    session.run();

    // Compute the proof.
    session.prove();
}

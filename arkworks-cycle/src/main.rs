#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

use ark_crypto_primitives::snark::{CircuitSpecificSetupSNARK, SNARK};
use ark_ec::pairing::Pairing;
use ark_ff::PrimeField;
use ark_groth16::VerifyingKey;
use ark_groth16::{prepare_verifying_key, Groth16, Proof};
use ark_relations::{
    lc,
    r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError},
};
use ark_std::{
    rand::{RngCore, SeedableRng},
    test_rng, UniformRand,
};
use rand::Rng;
use std::time::Instant;

use ark_bls12_381::Bls12_381;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Read, Write};
use ark_std::error::Error;
use ark_std::fs::File;
use ark_std::io;

mod graph_checks;
use crate::graph_checks::{
    check_multi_subgraph_topo_sort, check_subgraph_topo_sort, check_topo_sort,
};
use crate::graph_checks::{Boolean2DArray, Boolean3DArray, BooleanArray, Uint8Array};
// use crate::graph_checks::alloc;
use crate::graph_checks::hashing::hasher;
use ark_r1cs_std::alloc::AllocVar;
use ark_relations::r1cs::{ConstraintLayer, ConstraintSystem, TracingMode};

use crate::graph_checks::hashing::hasher_var;
use crate::graph_checks::hashing::matrix_flattener;
use crate::graph_checks::hashing::poseidon_parameters_for_test;
use ark_bls12_381::fr::Fr;
use ark_bls12_381::Config;
use ark_crypto_primitives::sponge::poseidon::PoseidonConfig;
use ark_ec::bls12::Bls12;
use ark_r1cs_std::eq::EqGadget;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::prelude::AllocationMode;
use ark_r1cs_std::R1CSVar;
use ark_relations::ns;
use ark_std::Zero;

fn main() {
    //function called by cargo run
    let adj_matrix = [
        [false, true, true, false],   //               [0]
        [false, false, true, false],  //               / \
        [false, false, false, true],  //             [1]->[2] -> 3
        [false, false, false, false], //
    ];
    let topological_sort = [0, 1, 2, 3];

    match test_prove_and_verify::<Bls12_381, 4>(adj_matrix, topological_sort) {
        Ok(()) => println!("finished successfully!"),
        Err(e) => eprintln!("Back in Main. Error: {:?}", e),
    }
}

// struct for generating the circuit trace
//the fields are the inputs to the circuit
struct MyGraphCircuitStruct<const N: usize, ConstraintF: PrimeField> {
    adj_matrix: [[bool; N]; N],
    toposort: [u8; N],
    adj_hash: ConstraintF,
}

// implementing cloning for MyGraphCircuitStruct
impl<const N: usize, ConstraintF: PrimeField> Clone for MyGraphCircuitStruct<N, ConstraintF> {
    fn clone(&self) -> Self {
        Self {
            adj_matrix: self.adj_matrix.clone(),
            toposort: self.toposort.clone(),
            adj_hash: self.adj_hash.clone(),
        }
    }
}

// Takes the struct that holds inputs and generates the entire circuit
impl<ConstraintF: PrimeField, const N: usize> ConstraintSynthesizer<ConstraintF>
    for MyGraphCircuitStruct<N, ConstraintF>
{
    fn generate_constraints(
        self,
        cs: ConstraintSystemRef<ConstraintF>,
    ) -> Result<(), SynthesisError> {
        // create input vars
        let adj_matrix_var =
            Boolean2DArray::new_witness(cs.clone(), || Ok(self.adj_matrix)).unwrap();
        let topo_var = Uint8Array::new_witness(cs.clone(), || Ok(self.toposort)).unwrap();
        let hash_claim_var: FpVar<ConstraintF> =
            FpVar::new_input(cs.clone(), || Ok(self.adj_hash))?;

        // check the claimed hash is correct
        let hash_real: &FpVar<ConstraintF> =
            &hasher_var::<N, ConstraintF>(cs.clone(), &adj_matrix_var).unwrap()[0];
        hash_real.enforce_equal(&hash_claim_var)?;

        // check the graph properties
        check_topo_sort(&adj_matrix_var, &topo_var).unwrap();

        // BENCHMARK 0: Circuit size
        println!("Number of constraints: {}", cs.num_constraints());
        Ok(())
    }
}

// Generate proof and write to file
fn write_proof_to_file(proof: &Proof<Bls12<Config>>, file_path: &str) -> Result<(), io::Error> {
    let mut compressed_bytes = Vec::new();
    proof.serialize_compressed(&mut compressed_bytes).unwrap();

    let mut file: File = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .open(file_path)
        .unwrap();
    file.write_all(&compressed_bytes)?;
    file.flush()?;
    Ok(())
}

// // Read proof from file

fn read_proof<E: Pairing>(file_path: &str) -> Result<Proof<E>, Box<dyn Error>> {
    // Open and read the file
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Deserialize the proof from the buffer
    let proof: Proof<E> = Proof::<E>::deserialize_compressed(&mut buffer.as_slice())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    return Ok(proof);
}

fn generate_graph<const N: usize>(allow_cycles: bool) -> ([[bool; N]; N], [u8; N]) {
    let mut rng = rand::thread_rng();
    let mut adj_matrix = [[false; N]; N];
    let mut topological_sort = [0u8; N];

    for i in 0..N {
        for j in 0..N {
            if allow_cycles && i != j {
                adj_matrix[i][j] = rng.gen();
            } else if j > i {
                adj_matrix[i][j] = rng.gen();
            }
        }
        topological_sort[i] = i as u8; // Convert usize to u8
    }

    // Note: The topological sort generated here may not be valid if the graph has cycles
    (adj_matrix, topological_sort)
}

// takes the adj matrix and toposort defined, builds the circuit, gens the proof, & verifies it
// also will write the proof and read the proof for I/O  demonstration
fn test_prove_and_verify<E: Pairing, const N: usize>(
    adj_matrix: [[bool; N]; N],
    topological_sort: [u8; N],
) -> Result<(), Box<dyn Error>> {
    // hardcoded for bls12_381 because our hash function is as well
    use ark_bls12_381::Config;
    use ark_bls12_381::Fr as PrimeField;
    use ark_ec::bls12::Bls12;

    let cs = ConstraintSystem::<Fr>::new_ref();
    //defining the inputs

    // Convert the adjacency matrix to Boolean2DArray
    let adj_matrix_boolean_2_d_array =
        Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix)).unwrap();
    let adj_hash_result = hasher(&adj_matrix_boolean_2_d_array);

    let adj_hash = match adj_hash_result {
        Ok(hash_vec) => hash_vec[0],
        Err(e) => return Err(Box::new(e)),
    };

    let circuit_inputs: MyGraphCircuitStruct<N, Fr> = MyGraphCircuitStruct {
        adj_matrix: adj_matrix,
        toposort: topological_sort,
        adj_hash: adj_hash,
    };

    // generate the proof
    let start_gen = Instant::now();
    let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(test_rng().next_u64());
    let (pk, vk) = Groth16::<Bls12_381>::setup(circuit_inputs.clone(), &mut rng).unwrap();
    let pvk = prepare_verifying_key::<Bls12_381>(&vk);
    let proof: Proof<Bls12<Config>> =
        Groth16::<Bls12_381>::prove(&pk, circuit_inputs, &mut rng).unwrap();
    let end_gen = start_gen.elapsed().as_millis();
    println!("Time taken to generate proof: {} ms", end_gen);
    // test some verification checks
    let start_verify = Instant::now();
    assert!(Groth16::<Bls12_381>::verify_with_processed_vk(&pvk, &[adj_hash], &proof).unwrap());
    let end_verify = start_verify.elapsed().as_millis();
    println!("Time take to verify proof: {} ms", end_verify);
    let false_hash = Fr::zero();
    assert!(!Groth16::<Bls12_381>::verify_with_processed_vk(&pvk, &[false_hash], &proof).unwrap());

    // test IO
    let file_path = "./proof.bin";
    write_proof_to_file(&proof, file_path)?;
    let read_proof: Proof<Bls12<Config>> = read_proof::<Bls12_381>(file_path)?;
    assert!(Groth16::<Bls12_381>::verify_with_processed_vk(&pvk, &[adj_hash], &proof).unwrap());

    Ok(())
}

#[test]
fn test_30_30_matrix() {
    let (adj_matrix, topological_sort) = generate_graph::<30>(false);

    // hardcoded for bls12_381 because our hash function is as well
    use ark_bls12_381::Config;
    use ark_bls12_381::Fr as PrimeField;
    use ark_ec::bls12::Bls12;

    let cs = ConstraintSystem::<Fr>::new_ref();
    //defining the inputs

    // Convert the adjacency matrix to Boolean2DArray
    let adj_matrix_boolean_2_d_array =
        Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix)).unwrap();
    let adj_hash_result = hasher(&adj_matrix_boolean_2_d_array);
    //adj_hash_result.clone().unwrap()
    let adj_hash = match adj_hash_result {
        Ok(hash_vec) => hash_vec[0],
        Err(e) => panic!("Error occurred: {:?}", e),
    };

    //BENCHMARK 1 : build the circuit
    let start_circ = Instant::now();
    let circuit_inputs: MyGraphCircuitStruct<30, Fr> = MyGraphCircuitStruct {
        adj_matrix: adj_matrix,
        toposort: topological_sort,
        adj_hash: adj_hash,
    };
    let end_circ = start_circ.elapsed().as_millis();
    println!("Time taken to build circuit: {} ms", end_circ);

    // BENCHMARK 2: generate the proof
    let start_gen = Instant::now();
    let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(test_rng().next_u64());
    let (pk, vk) = Groth16::<Bls12_381>::setup(circuit_inputs.clone(), &mut rng).unwrap();
    let pvk = prepare_verifying_key::<Bls12_381>(&vk);
    let proof: Proof<Bls12<Config>> =
        Groth16::<Bls12_381>::prove(&pk, circuit_inputs, &mut rng).unwrap();
    let end_gen = start_gen.elapsed().as_millis();
    println!("Time taken to generate proof: {} ms", end_gen);

    // BENCHMARK 3: Write proof to file
    let start_write = Instant::now();
    let file_path = "./proof.bin";
    write_proof_to_file(&proof, file_path).unwrap();
    let end_write = start_write.elapsed().as_millis();
    println!("Time taken to write proof to binary: {} ms", end_write);

    //BENCHMARK 3: verify the proof
    let start_verify = Instant::now();
    assert!(Groth16::<Bls12_381>::verify_with_processed_vk(&pvk, &[adj_hash], &proof).unwrap());
    let end_verify = start_verify.elapsed().as_millis();
    println!("Time take to verify proof: {} ms", end_verify);
}

#[test]
fn test_35_35_matrix() {
    let (adj_matrix, topological_sort) = generate_graph::<35>(false);

    // hardcoded for bls12_381 because our hash function is as well
    use ark_bls12_381::Config;
    use ark_bls12_381::Fr as PrimeField;
    use ark_ec::bls12::Bls12;

    let cs = ConstraintSystem::<Fr>::new_ref();
    //defining the inputs

    // Convert the adjacency matrix to Boolean2DArray
    let adj_matrix_boolean_2_d_array =
        Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix)).unwrap();
    let adj_hash_result = hasher(&adj_matrix_boolean_2_d_array);
    //adj_hash_result.clone().unwrap()
    let adj_hash = match adj_hash_result {
        Ok(hash_vec) => hash_vec[0],
        Err(e) => panic!("Error occurred: {:?}", e),
    };

    //BENCHMARK 1 : build the circuit
    let start_circ = Instant::now();
    let circuit_inputs: MyGraphCircuitStruct<35, Fr> = MyGraphCircuitStruct {
        adj_matrix: adj_matrix,
        toposort: topological_sort,
        adj_hash: adj_hash,
    };
    let end_circ = start_circ.elapsed().as_millis();
    println!("Time taken to build circuit: {} ms", end_circ);

    // BENCHMARK 2: generate the proof
    let start_gen = Instant::now();
    let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(test_rng().next_u64());
    let (pk, vk) = Groth16::<Bls12_381>::setup(circuit_inputs.clone(), &mut rng).unwrap();
    let pvk = prepare_verifying_key::<Bls12_381>(&vk);
    let proof: Proof<Bls12<Config>> =
        Groth16::<Bls12_381>::prove(&pk, circuit_inputs, &mut rng).unwrap();
    let end_gen = start_gen.elapsed().as_millis();
    println!("Time taken to generate proof: {} ms", end_gen);

    // BENCHMARK 3: Write proof to file
    let start_write = Instant::now();
    let file_path = "./proof.bin";
    write_proof_to_file(&proof, file_path).unwrap();
    let end_write = start_write.elapsed().as_millis();
    println!("Time taken to write proof to binary: {} ms", end_write);

    //BENCHMARK 3: verify the proof
    let start_verify = Instant::now();
    assert!(Groth16::<Bls12_381>::verify_with_processed_vk(&pvk, &[adj_hash], &proof).unwrap());
    let end_verify = start_verify.elapsed().as_millis();
    println!("Time take to verify proof: {} ms", end_verify);
}

#[test]
fn test_40_40_matrix() {
    let (adj_matrix, topological_sort) = generate_graph::<40>(false);

    // hardcoded for bls12_381 because our hash function is as well
    use ark_bls12_381::Config;
    use ark_bls12_381::Fr as PrimeField;
    use ark_ec::bls12::Bls12;

    let cs = ConstraintSystem::<Fr>::new_ref();
    //defining the inputs

    // Convert the adjacency matrix to Boolean2DArray
    let adj_matrix_boolean_2_d_array =
        Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix)).unwrap();
    let adj_hash_result = hasher(&adj_matrix_boolean_2_d_array);
    //adj_hash_result.clone().unwrap()
    let adj_hash = match adj_hash_result {
        Ok(hash_vec) => hash_vec[0],
        Err(e) => panic!("Error occurred: {:?}", e),
    };

    //BENCHMARK 1 : build the circuit
    let start_circ = Instant::now();
    let circuit_inputs: MyGraphCircuitStruct<40, Fr> = MyGraphCircuitStruct {
        adj_matrix: adj_matrix,
        toposort: topological_sort,
        adj_hash: adj_hash,
    };
    let end_circ = start_circ.elapsed().as_millis();
    println!("Time taken to build circuit: {} ms", end_circ);

    // BENCHMARK 2: generate the proof
    let start_gen = Instant::now();
    let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(test_rng().next_u64());
    let (pk, vk) = Groth16::<Bls12_381>::setup(circuit_inputs.clone(), &mut rng).unwrap();
    let pvk = prepare_verifying_key::<Bls12_381>(&vk);
    let proof: Proof<Bls12<Config>> =
        Groth16::<Bls12_381>::prove(&pk, circuit_inputs, &mut rng).unwrap();
    let end_gen = start_gen.elapsed().as_millis();
    println!("Time taken to generate proof: {} ms", end_gen);

    // BENCHMARK 3: Write proof to file
    let start_write = Instant::now();
    let file_path = "./proof.bin";
    write_proof_to_file(&proof, file_path).unwrap();
    let end_write = start_write.elapsed().as_millis();
    println!("Time taken to write proof to binary: {} ms", end_write);

    //BENCHMARK 3: verify the proof
    let start_verify = Instant::now();
    assert!(Groth16::<Bls12_381>::verify_with_processed_vk(&pvk, &[adj_hash], &proof).unwrap());
    let end_verify = start_verify.elapsed().as_millis();
    println!("Time take to verify proof: {} ms", end_verify);
}

#[test]
fn test_50_50_matrix() {
    let (adj_matrix, topological_sort) = generate_graph::<45>(false);

    // hardcoded for bls12_381 because our hash function is as well
    use ark_bls12_381::Config;
    use ark_bls12_381::Fr as PrimeField;
    use ark_ec::bls12::Bls12;

    let cs = ConstraintSystem::<Fr>::new_ref();
    //defining the inputs

    // Convert the adjacency matrix to Boolean2DArray
    let adj_matrix_boolean_2_d_array =
        Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix)).unwrap();
    let adj_hash_result = hasher(&adj_matrix_boolean_2_d_array);
    //adj_hash_result.clone().unwrap()
    let adj_hash = match adj_hash_result {
        Ok(hash_vec) => hash_vec[0],
        Err(e) => panic!("Error occurred: {:?}", e),
    };

    //BENCHMARK 1 : build the circuit
    let start_circ = Instant::now();
    let circuit_inputs: MyGraphCircuitStruct<45, Fr> = MyGraphCircuitStruct {
        adj_matrix: adj_matrix,
        toposort: topological_sort,
        adj_hash: adj_hash,
    };
    let end_circ = start_circ.elapsed().as_millis();
    println!("Time taken to build circuit: {} ms", end_circ);

    // BENCHMARK 2: generate the proof
    let start_gen = Instant::now();
    let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(test_rng().next_u64());
    let (pk, vk) = Groth16::<Bls12_381>::setup(circuit_inputs.clone(), &mut rng).unwrap();
    let pvk = prepare_verifying_key::<Bls12_381>(&vk);
    let proof: Proof<Bls12<Config>> =
        Groth16::<Bls12_381>::prove(&pk, circuit_inputs, &mut rng).unwrap();
    let end_gen = start_gen.elapsed().as_millis();
    println!("Time taken to generate proof: {} ms", end_gen);

    // BENCHMARK 3: Write proof to file
    let start_write = Instant::now();
    let file_path = "./proof.bin";
    write_proof_to_file(&proof, file_path).unwrap();
    let end_write = start_write.elapsed().as_millis();
    println!("Time taken to write proof to binary: {} ms", end_write);

    //BENCHMARK 3: verify the proof
    let start_verify = Instant::now();
    assert!(Groth16::<Bls12_381>::verify_with_processed_vk(&pvk, &[adj_hash], &proof).unwrap());
    let end_verify = start_verify.elapsed().as_millis();
    println!("Time take to verify proof: {} ms", end_verify);
}

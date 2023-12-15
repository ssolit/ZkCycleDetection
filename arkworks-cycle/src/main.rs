#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

use ark_crypto_primitives::snark::{CircuitSpecificSetupSNARK, SNARK};
use ark_ec::pairing::Pairing;
use ark_ff::PrimeField;
use std::time::Instant;

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





use ark_bls12_381::fr::Fr;
use crate::graph_checks::hashing::poseidon_parameters_for_test;
use crate::graph_checks::hashing::matrix_flattener;
use ark_crypto_primitives::sponge::poseidon::{PoseidonConfig};
use ark_r1cs_std::fields::fp::FpVar;
use ark_relations::ns;



fn main() {
    //function called by cargo run
    match test_prove_and_verify::<Bls12_381>() {
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
        let adj_matrix_var =
            Boolean2DArray::new_witness(cs.clone(), || Ok(self.adj_matrix)).unwrap();
        let topo_var = Uint8Array::new_witness(cs.clone(), || Ok(self.toposort)).unwrap();



        // TODO: Get the public input hash;
        // let mut constraint_sponge = PoseidonSpongeVar::<Fr>::new(cs.clone(), &p_config);
        // let flattened_matrix = matrix_flattener::<N, ConstraintF>(&adj_matrix_var);
        
        // let cs = ConstraintSystem::<Fr>::new_ref();
        // let mut rng = test_rng();
        // let absorb1: Vec<_> = (0..256).map(|_| ConstraintF::rand(&mut rng)).collect();
        // let hash_vec = vec![self.adj_hash];
        // let absorb1_var: Vec<_> = hash_vec
        //     .iter()
        //     .map(|v| FpVar::new_input(ns!(cs, "absorb1"), || Ok(*v)).unwrap())
        //     .collect();

        // let my_var = FpVar::new_input(ns!(cs, "my_var"), || Ok(*self.adj_hash[0])).unwrap();

        // let adj_hash_var = cs.new_input_variable(|| Ok(self.adj_hash))?;
        // let input_var = ConstraintF::new_variable(
        //     ns!(cs, "input_commitment"),
        //     || Ok(self.adj_hash),
        //     AllocationMode::Input,
        // )?;
        let zero : ConstraintF = ConstraintF::zero();
        let adj_hash_var = zero + self.adj_hash;
        



        check_topo_sort(&adj_matrix_var, &topo_var, &adj_hash_var).unwrap();
        println!("Number of constraints: {}", cs.num_constraints());

        Ok(())
    }
}

//takes the adj matrix and toposort defined, builds the circuit, gens the proof, & verifies it
//also will write the proof and read the proof for I/O  demonstration
fn test_prove_and_verify<E>() -> Result<(), Box<dyn Error>>
where
    E: Pairing,
{
    use ark_bls12_381::Fr;
    let cs = ConstraintSystem::<Fr>::new_ref();
    //defining the inputs
    let adj_matrix = [
        [false, true, true, false],   //               [0]
        [false, false, true, false],  //               / \
        [false, false, false, true],  //             [1]->[2] -> 3
        [false, false, false, false], //
    ];
    let topological_sort = [0, 1, 2, 3];

    // Convert the adjacency matrix to Boolean2DArray
    let adj_matrix_boolean_2_d_array =
        Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix)).unwrap();
    let adj_hash_result = hasher(&adj_matrix_boolean_2_d_array);

    let adj_hash = match adj_hash_result {
        Ok(hash_vec) => hash_vec[0],
        Err(e) => return Err(Box::new(e)),
    };

    let circuit_inputs: MyGraphCircuitStruct<4, Fr> = MyGraphCircuitStruct {
        adj_matrix: adj_matrix,
        toposort: topological_sort,
        adj_hash: adj_hash,
    };
    //TODO: Unwrap circuit_inputs to just grab the adj_matrix
    // Create a constraint system
  
    

    

    // generate the proof
    let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(test_rng().next_u64());
    let (pk, vk) = Groth16::<Bls12_381>::setup(circuit_inputs.clone(), &mut rng).unwrap();
    let pvk = prepare_verifying_key::<Bls12_381>(&vk);
    let proof = Groth16::<Bls12_381>::prove(&pk, circuit_inputs, &mut rng).unwrap();

    // assert!(Groth16::<Bls12_381>::verify_with_processed_vk(&pvk, &adj_hash[..], &proof).unwrap());
    assert!(Groth16::<Bls12_381>::verify_with_processed_vk(&pvk, &[adj_hash], &proof).unwrap());

    //TODO: Make failing test case with wrong hash
    let false_adj_matrix = [
        [false, true, true, false],  //               [0]
        [false, false, true, false], //               / \
        [false, false, false, true], //             [1]->[2] -> 3
        [false, false, true, false], //
    ];

    // let false_hash = hasher(&false_adj_matrix);
    // assert!(!Groth16::<E>::verify_with_processed_vk(&pvk, &false_hash, &proof).unwrap());

    //TODO: Make read and write functions with this code (I/O functions)
    // let mut compressed_bytes = Vec::new();
    // proof.serialize_compressed(&mut compressed_bytes).unwrap();
    // let file_path = "./proof.bin";
    // let mut file: File = std::fs::OpenOptions::new()
    //     .create(true)
    //     .write(true)
    //     .read(true)
    //     .open(file_path)
    //     .unwrap();
    // file.write_all(&compressed_bytes)?;
    // file.flush()?;

    // let mut file2 = File::open(file_path)?;
    // let mut buffer = Vec::new();
    // file2.read_to_end(&mut buffer)?;
    // let read_proof = Proof::<E>::deserialize_compressed(&mut buffer.as_slice())?;

    // //checking I/O  was done correctly
    // assert!(Groth16::<E>::verify_with_processed_vk(&pvk, &[], &read_proof).unwrap());
    // // assert!(!Groth16::<E>::verify_with_processed_vk(&pvk, &[a], &read_proof).unwrap());
    // Generate and write proof
    // Example usage with specified types and size N
    // let start = Instant::now();
    // write_proof_to_file::<Bls12_381, 4>(&adj_matrix, &topological_sort, "./proof.bin", &adj_hash)?;
    // let duration = start.elapsed();
    // println!("Execution time: {:?}", duration);

    // Read and verify proof
    // let is_valid = read_and_verify_proof::<Bls12_381>("./proof.bin", &pvk, &[])?;
    // assert!(Groth16::<E>::verify_with_processed_vk(&pvk, &[], &read_proof).unwrap());

    Ok(())
}

// // Generate proof and write to file
// fn write_proof_to_file<E: Pairing, const N: usize>(
//     adj_matrix: &[[bool; N]; N],
//     toposort: &[u8; N],
//     file_path: &str,
//     adj_hash: &Fr,
// ) -> Result<(), io::Error> {
//     //create circuit inputs struct:
//     let circuit_inputs: MyGraphCircuitStruct<N, ConstraintF> = MyGraphCircuitStruct {
//         adj_matrix: *adj_matrix,
//         toposort: *toposort,
//         adj_hash: *adj_hash,
//     };

//     // Generate the proof using the circuit and inputs
//     let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(test_rng().next_u64());
//     let (pk, vk) = Groth16::<E>::setup(circuit_inputs.clone(), &mut rng).unwrap();
//     let pvk = prepare_verifying_key::<E>(&vk);
//     let proof = Groth16::<E>::prove(&pk, circuit_inputs, &mut rng).unwrap();

//     // Serialize the proof to a byte vector

//     let mut compressed_bytes = Vec::new();
//     proof.serialize_compressed(&mut compressed_bytes).unwrap();

//     // Create and write the proof to the file

//     let mut file = File::create(file_path)?;
//     file.write_all(&compressed_bytes)?;
//     file.flush()?;

//     Ok(())
// }

// // Read proof from file

// fn read_and_verify_proof<E: Pairing>(
//     file_path: &str,
//     pvk: &VerifyingKey<E>,
//     public_input: &[E::ScalarField],
// ) -> Result<bool, Box<dyn Error>> {
//     // Open and read the file
//     let mut file = File::open(file_path)?;
//     let mut buffer = Vec::new();
//     file.read_to_end(&mut buffer)?;

//     // Deserialize the proof from the buffer
//     let proof = Proof::<E>::deserialize_compressed(&mut buffer.as_slice())
//         .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

//     // Verify the proof
//     Groth16::<E>::verify(pvk, public_input, &proof).map_err(|e| Box::new(e) as Box<dyn Error>)
// }

// #![allow(unused_variables)]
// #![allow(unused_imports)]
// #![allow(dead_code)]

use ark_crypto_primitives::snark::{CircuitSpecificSetupSNARK, SNARK};
use ark_ff::PrimeField;
use ark_groth16::{prepare_verifying_key, Groth16, Proof};
use ark_relations::{
    r1cs::{ConstraintSystem, ConstraintSystemRef, ConstraintSynthesizer, SynthesisError},
};
use ark_std::{
    rand::{RngCore, SeedableRng},
    test_rng,
};
use ark_bls12_381::{
    Bls12_381,
    fr::Fr,
    Config,
};
use ark_ec::bls12::Bls12;
use ark_ec::pairing::Pairing;

use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Read, Write};
use ark_std::{
    error::Error,
    fs::File,
    io,
    Zero
};

// use crate::hashing::poseidon_parameters_for_test;
use crate::hashing::{hasher_var, hasher};
use ark_r1cs_std::{
    fields::fp::FpVar,
    eq::EqGadget,
    alloc::AllocVar,
};

mod utils;
mod graph_checks;
mod hashing;

use crate::graph_checks::{
    check_topo_sort,
    // check_subgraph_topo_sort, 
    // check_multi_subgraph_topo_sort, 
};
use crate::utils::{
    Boolean2DArray,
    Uint8Array
    // Boolean3DArray, 
    // BooleanArray, 
};





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
        let hash_claim_var: FpVar<ConstraintF> = FpVar::new_input(cs.clone(), || Ok(self.adj_hash))?;

        // check the claimed hash is correct
        let hash_real: &FpVar<ConstraintF> = &hasher_var::<N, ConstraintF>(cs.clone(), &adj_matrix_var).unwrap()[0];
        hash_real.enforce_equal(&hash_claim_var)?;

        // check the graph properties
        check_topo_sort(&adj_matrix_var, &topo_var).unwrap();

        // finish
        println!("Number of constraints: {}", cs.num_constraints());
        Ok(())
    }
}



//takes the adj matrix and toposort defined, builds the circuit, gens the proof, & verifies it
//also will write the proof and read the proof for I/O  demonstration
// hardcoded for bls12_381 because our hash function is as well
fn test_prove_and_verify<E: Pairing, const N: usize>(
    adj_matrix: [[bool; N]; N], 
    topological_sort: [u8; N],
) -> Result<(), Box<dyn Error>> {

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
    let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(test_rng().next_u64());
    let (pk, vk) = Groth16::<Bls12_381>::setup(circuit_inputs.clone(), &mut rng).unwrap();
    let pvk = prepare_verifying_key::<Bls12_381>(&vk);
    let proof: Proof<Bls12<Config>> = Groth16::<Bls12_381>::prove(&pk, circuit_inputs, &mut rng).unwrap();

    // test some verification checks
    assert!(Groth16::<Bls12_381>::verify_with_processed_vk(&pvk, &[adj_hash], &proof).unwrap());
    let false_hash = Fr::zero();
    assert!(!Groth16::<Bls12_381>::verify_with_processed_vk(&pvk, &[false_hash], &proof).unwrap());

    // test IO
    let file_path = "./proof.bin";
    write_proof_to_file(&proof, file_path)?;
    let read_proof: Proof<Bls12<Config>> = read_proof::<Bls12_381>(file_path)?;
    assert!(Groth16::<Bls12_381>::verify_with_processed_vk(&pvk, &[adj_hash], &read_proof).unwrap());

    Ok(())
}

// Generate proof and write to file
fn write_proof_to_file(
    proof: &Proof<Bls12<Config>>,
    file_path: &str,
) -> Result<(), io::Error> {
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

fn read_proof<E: Pairing>(
    file_path: &str,
) -> Result<Proof<E>, Box<dyn Error>> {
    // Open and read the file
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Deserialize the proof from the buffer
    let proof: Proof<E> = Proof::<E>::deserialize_compressed(&mut buffer.as_slice())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    return Ok(proof)
}

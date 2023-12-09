#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

use ark_groth16::{Groth16, prepare_verifying_key, Proof};
use ark_crypto_primitives::snark::{CircuitSpecificSetupSNARK, SNARK};
use ark_ec::pairing::Pairing;
use ark_ff::PrimeField;
use ark_relations::{
    lc,
    r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError},
};
use ark_std::{
    rand::{RngCore, SeedableRng},
    test_rng, UniformRand,
};

use ark_serialize::{Read, Write, CanonicalSerialize, CanonicalDeserialize};
use ark_std::io;
use ark_std::fs::File;
use ark_std::error::Error;
use ark_bls12_381::Bls12_381;

mod graph_checks;
use crate::graph_checks::{Uint8Array, BooleanArray, Boolean2DArray, Boolean3DArray};
use crate::graph_checks::{check_topo_sort, check_subgraph_topo_sort, check_multi_subgraph_topo_sort};
// use crate::graph_checks::alloc;

use ark_relations::r1cs::{ConstraintLayer, ConstraintSystem, TracingMode};
use ark_r1cs_std::alloc::AllocVar;


fn main() {
    match test_prove_and_verify::<Bls12_381>() {
        Ok(()) => println!("finished successfully!"),
        Err(e) => eprintln!("Back in Main. Error: {:?}", e),
    }
}


struct MyGraphCircuitStruct<const N: usize>{
    adj_matrix: [[bool; N]; N], 
    toposort: [u8; N],
}

impl<const N: usize> Clone for MyGraphCircuitStruct<N> {
    fn clone(&self) -> Self {
        Self {
            adj_matrix: self.adj_matrix.clone(),
            toposort: self.toposort.clone(),
        }
    }
}

impl<ConstraintF: PrimeField, const N: usize> ConstraintSynthesizer<ConstraintF> for MyGraphCircuitStruct<N> {
    fn generate_constraints(
        self,
        cs: ConstraintSystemRef<ConstraintF>,
    ) -> Result<(), SynthesisError> {
        let adj_matrix_var = Boolean2DArray::new_witness(cs.clone(), || Ok(self.adj_matrix)).unwrap();
        let topo_var = Uint8Array::new_witness(cs.clone(), || Ok(self.toposort)).unwrap();
        check_topo_sort(&adj_matrix_var, &topo_var).unwrap();
        Ok(())
    }
}


// fn set_up_constraints<ConstraintF: ark_ff::Field + ark_ff::PrimeField>(cs: ConstraintSystemRef<ConstraintF>,) {
//     // Check that it accepts a valid solution.
//     let adj_matrix = [                                      
//         [false, true, true, false],  //               [0]
//         [false, false, true, false], //               / \
//         [false, false, false, true], //             [1]->[2] -> 3
//         [false, false, false, false] //                     
//     ];
//     let topo = [0, 1, 2, 3];
    
//     let adj_matrix_var = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix)).unwrap();
//     let topo_var = Uint8Array::new_witness(cs.clone(), || Ok(topo)).unwrap();
//     check_topo_sort(&adj_matrix_var, &topo_var).unwrap();
// }

fn test_prove_and_verify<E>() -> Result<(), Box<dyn Error>>
where
    E: Pairing,
{
    // // define the circuit and inputs
    // use ark_bls12_381::Fq as F;
    // let cs = ConstraintSystem::<F>::new_ref();

    let adj_matrix = [                                      
        [false, true, true, false],  //               [0]
        [false, false, true, false], //               / \
        [false, false, false, true], //             [1]->[2] -> 3
        [false, false, false, false] //                     
    ];
    let topological_sort = [0, 1, 2, 3];
    let circuit_inputs: MyGraphCircuitStruct<4> = MyGraphCircuitStruct {
        adj_matrix: adj_matrix,
        toposort: topological_sort
    };

    // generate the proof
    let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(test_rng().next_u64());
    let (pk, vk) = Groth16::<E>::setup(circuit_inputs.clone(), &mut rng).unwrap();
    let pvk = prepare_verifying_key::<E>(&vk);
    let proof = Groth16::<E>::prove(
        &pk,
        circuit_inputs,
        &mut rng,
    )
    .unwrap();

    assert!(Groth16::<E>::verify_with_processed_vk(&pvk, &[], &proof).unwrap());
    // assert!(!Groth16::<E>::verify_with_processed_vk(&pvk, &[a], &proof).unwrap());

    let mut compressed_bytes = Vec::new();
    proof.serialize_compressed(&mut compressed_bytes).unwrap();
    let file_path = "./proof.bin";
    let mut file: File = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .open(file_path)
        .unwrap();
    file.write_all(&compressed_bytes)?;
    file.flush()?;

    let mut file2 = File::open(file_path)?;
    let mut buffer = Vec::new();
    file2.read_to_end(&mut buffer)?;
    let read_proof = Proof::<E>::deserialize_compressed(&mut buffer.as_slice())?;

    assert!(Groth16::<E>::verify_with_processed_vk(&pvk, &[], &read_proof).unwrap());
    // assert!(!Groth16::<E>::verify_with_processed_vk(&pvk, &[a], &read_proof).unwrap());

    Ok(())
}

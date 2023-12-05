#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]



use ark_ff::PrimeField;
use ark_r1cs_std::{
    prelude::{Boolean, AllocVar},
    uint8::UInt8
};

pub struct Uint8Array<const N: usize, ConstraintF: PrimeField>([UInt8<ConstraintF>; N]);
pub struct BooleanArray<const N: usize, ConstraintF: PrimeField>([Boolean<ConstraintF>; N]);
pub struct Boolean2DArray<const N: usize, ConstraintF: PrimeField>([[Boolean<ConstraintF>; N]; N]);
pub struct Boolean3DArray<const N: usize, const M: usize, ConstraintF: PrimeField>([[[Boolean<ConstraintF>; N]; N]; M]);

mod cmp;
mod alloc;
mod graph_checks;

use crate::graph_checks::graph_checks::{check_topo_sort, check_subgraph_topo_sort, check_multi_subgraph_topo_sort};


// use ark_groth16::Groth16;
// use ark_bls12_381::{Bls12_381, Fr as BlsFr};
// use ark_std::{ops::*, UniformRand};
// use ark_relations::{
// 	lc,
// 	r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError},
// };
// use ark_snark::SNARK;


use ark_bls12_381::{Bls12_381, Fr as BlsFr};
use ark_ed_on_bn254::{EdwardsAffine, Fr as BabyJubJub};
use ark_groth16::Groth16;
use ark_marlin::Marlin;
use ark_poly::univariate::DensePolynomial;
use ark_poly_commit::{ipa_pc::InnerProductArgPC, marlin_pc::MarlinKZG10};
use ark_snark::SNARK;
use ark_std::{ops::*, UniformRand};
use blake2::Blake2s;
use ark_relations::{
	lc,
	r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError},
};
use ark_crypto_primitives::{sponge};






fn main() {
    // TODO: add IO?
}

/// Defines `DummyCircuit`
#[derive(Copy)]
struct DummyCircuit<F: PrimeField> {
	pub a: Option<F>,
	pub b: Option<F>,
	pub num_variables: usize,
	pub num_constraints: usize,
}

/// constructor for DummyCircuit
impl<F: PrimeField> Clone for DummyCircuit<F> {
	fn clone(&self) -> Self {
		DummyCircuit {
			a: self.a,
			b: self.b,
			num_variables: self.num_variables,
			num_constraints: self.num_constraints,
		}
	}
}

/// Implementation of the `ConstraintSynthesizer` trait for the `DummyCircuit`
/// https://github.com/arkworks-rs/snark/blob/master/relations/src/r1cs/constraint_system.rs
///
/// This is the main function that is called by the `R1CS` library to generate
/// the constraints for the `DummyCircuit`.
impl<F: PrimeField> ConstraintSynthesizer<F> for DummyCircuit<F> {
	fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
		let a = cs.new_witness_variable(|| self.a.ok_or(SynthesisError::AssignmentMissing))?;
		let b = cs.new_witness_variable(|| self.b.ok_or(SynthesisError::AssignmentMissing))?;
		let c = cs.new_input_variable(|| {
			let a = self.a.ok_or(SynthesisError::AssignmentMissing)?;
			let b = self.b.ok_or(SynthesisError::AssignmentMissing)?;

			Ok(a * b)
		})?;

		for _ in 0..self.num_constraints {
			cs.enforce_constraint(lc!() + a, lc!() + b, lc!() + c)?;
		}

		Ok(())
	}
}


// fn groth16_test() {
//     // let adj_matrix = [                                      
//     //     [false, true, true, false],  //               [0]
//     //     [false, false, true, false], //               / \
//     //     [false, false, false, true], //             [1]->[2] -> 3
//     //     [false, false, false, false] //                     
//     // ];
//     // let topo = [0, 1, 2, 3];

//     // let params
//     let rng = &mut ark_std::test_rng();
//     let c = DummyCircuit::<BlsFr> {
//         a: Some(BlsFr::rand(rng)),
//         b: Some(BlsFr::rand(rng)),
//         num_variables: 0,
//         num_constraints: 3,
//     };
    

//     let (pk, vk) = Groth16::<Bls12_381>::circuit_specific_setup(c, rng).unwrap();
//     let proof = Groth16::<Bls12_381>::prove(&pk, c.clone(), rng).unwrap();

//     let v = c.a.unwrap().mul(c.b.unwrap());

//     let res = Groth16::<Bls12_381>::verify(&vk, &vec![v], &proof).unwrap();
//     assert!(res);
// }





fn marlin_test() {
    let rng = &mut ark_std::test_rng();

    let nc = 3;
    let nv = 3;
    let c = DummyCircuit::<BlsFr> {
        a: Some(BlsFr::rand(rng)),
        b: Some(BlsFr::rand(rng)),
        num_variables: nv,
        num_constraints: nc,
    };

    let spongee = sponge::new();

    type KZG10 = MarlinKZG10<Bls12_381, DensePolynomial<BlsFr>, spongee>;
    // type MarlinSetup = Marlin<BlsFr, KZG10, Blake2s>;

    // let srs = MarlinSetup::universal_setup(nc, nv, nv, rng).unwrap();
    // let (pk, vk) = MarlinSetup::index(&srs, c).unwrap();
    // let proof = MarlinSetup::prove(&pk, c.clone(), rng).unwrap();

    // let v = c.a.unwrap().mul(c.b.unwrap());

    // let res = MarlinSetup::verify(&vk, &vec![v], &proof, rng).unwrap();
    // assert!(res);
}
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
// use ark_ff::UniformRand;


fn main() {
    // TODO: add IO?
}

fn groth16_test() {
    let adj_matrix = [                                      
        [false, true, true, false],  //               [0]
        [false, false, true, false], //               / \
        [false, false, false, true], //             [1]->[2] -> 3
        [false, false, false, false] //                     
    ];
    let topo = [0, 1, 2, 3];

    // // let params
    // let c = DummyCircuit::<BlsFr> {
    //     a: Some(BlsFr::rand(rng)),
    //     b: Some(BlsFr::rand(rng)),
    //     num_variables: 0,
    //     num_constraints: 3,
    // };

    // let (pk, vk) = Groth16::<Bls12_381>::circuit_specific_setup(c, rng).unwrap();
    // let proof = Groth16::<Bls12_381>::prove(&pk, c.clone(), rng).unwrap();

    // let v = c.a.unwrap().mul(c.b.unwrap());

    // let res = Groth16::<Bls12_381>::verify(&vk, &vec![v], &proof).unwrap();
    // assert!(res);
}






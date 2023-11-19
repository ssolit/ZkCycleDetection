#![allow(unused_imports)]
#![allow(dead_code)]

use ark_ff::PrimeField;
use ark_r1cs_std::{
    prelude::{Boolean, EqGadget, AllocVar},
    uint8::UInt8
};
use ark_relations::r1cs::{SynthesisError, ConstraintSystem};
use cmp::CmpGadget;

mod cmp;
mod alloc;

pub struct AdjMatrix<const N: usize, ConstraintF: PrimeField>([[Boolean<ConstraintF>; N]; N]);
pub struct TopoSort<const N: usize, ConstraintF: PrimeField>([UInt8<ConstraintF>; N]);

fn check_topo_sort<const N: usize, ConstraintF: PrimeField>(
    adj_matrix: &AdjMatrix<N, ConstraintF>, 
    topo: &TopoSort<N, ConstraintF>,
) -> Result<(), SynthesisError> {
    for i in 0..N {
        for j in i+1..N {
            let transacted = &adj_matrix.0[i][j]; // true if person i sent to person j
            let wrong_order = topo.0[i].is_gt(&topo.0[j])?; // i is later in the topo sort than j 
            transacted.and(&wrong_order)?.enforce_equal(&Boolean::TRUE)?;
        }
    }
    Ok(())
}

fn check_helper<const N: usize, ConstraintF: PrimeField>(
    adj_matrix: &[[bool; N]; N],
    topo: &[u8; N],
) {
    let cs = ConstraintSystem::<ConstraintF>::new_ref();
    let adj_matrix_var = AdjMatrix::new_witness(cs.clone(), || Ok(adj_matrix)).unwrap();
    let topo_var = TopoSort::new_witness(cs.clone(), || Ok(topo)).unwrap();
    check_topo_sort(&adj_matrix_var, &topo_var).unwrap();
    // //TODO: check hash of adj_matrix matches some public input
    assert!(cs.is_satisfied().unwrap());
}

fn main() {
    use ark_bls12_381::Fq as F;
    // Check that it accepts a valid solution.
    let adj_matrix = [
        [false, true],
        [false, false],
    ];
    let topo = [0, 1];
    check_helper::<2, F>(&adj_matrix, &topo);

    // Check that it rejects a solution with a repeated number in a row.
    // let adj_matrix = [
    //     [false, true],
    //     [false, false],
    // ];
    // let topo = [1, 0];
    // check_helper::<2, F>(&adj_matrix, &topo);
}

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
    adjMatrix: &AdjMatrix<N, ConstraintF>, 
    topo: &TopoSort<N, ConstraintF>,
) -> Result<(), SynthesisError> {
    for i in 0..N {
        for j in i+1..N {
            let value = topo.0[i].value;
            // adjMatrix.0[topo.0[i].value][topo.0[j]].enforce_equal(&Boolean::FALSE)?;
        }
    }
    // let c = adjMatrix.0;
    // c[0][0].enforce_equal(&Boolean::FALSE)?;
    // for (i, cell) in topo.iter().enumerate() {
    //     for (cell2) in topo[i+1..]

    // }
    Ok(())
}

fn check_helper<const N: usize, ConstraintF: PrimeField>(
    adjMatrix: &[[bool; N]; N],
    topo: &[u8; N],
) {
    // let cs = ConstraintSystem::<ConstraintF>::new_ref();
    // let adjMatrix_var = AdjMatrix::new_witness(cs.clone(), || Ok(adjMatrix)).unwrap();
    // let topo_var = TopoSort::new_witness(cs.clone(), || Ok(topo)).unwrap();
    // check_topo_sort(&adjMatrix_var, &topo_var).unwrap();
    // //TODO: check hash of adjMatrix matches some public input
    // assert!(cs.is_satisfied().unwrap());
}

fn main() {
    use ark_bls12_381::Fq as F;
    // Check that it accepts a valid solution.
    let adjMatrix = [
        [false, true],
        [false, false],
    ];
    let topo = [0, 1];
    check_helper::<2, F>(&adjMatrix, &topo);

    // Check that it rejects a solution with a repeated number in a row.
    // let adjMatrix = [
    //     [false, true],
    //     [false, false],
    // ];
    // let topo = [1, 0];
    // check_helper::<2, F>(&adjMatrix, &topo);
}

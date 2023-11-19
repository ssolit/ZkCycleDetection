use std::borrow::Borrow;

use ark_ff::PrimeField;
use ark_r1cs_std::{prelude::{AllocVar, AllocationMode}, uint8::UInt8};
use ark_relations::r1cs::{Namespace, SynthesisError};

use crate::{AdjMatrix};

impl<const N: usize, F: PrimeField> AllocVar<[[u8; N]; N], F> for AdjMatrix<N, F> {
    fn new_variable<T: Borrow<[[u8; N]; N]>>(
        cs: impl Into<Namespace<F>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let cs = cs.into();
        let row = [(); N].map(|_| Boolean::constant(0));
        let mut adjMatrix = AdjMatrix([(); N].map(|_| row.clone()));
        let value = f().map_or([[0; N]; N], |f| *f.borrow());
        for (i, row) in value.into_iter().enumerate() {
            for (j, cell) in row.into_iter().enumerate() {
                adjMatrix.0[i][j] = Boolean::new_variable(cs.clone(), || Ok(cell), mode)?;
            }
        }
        Ok(adjMatrix)
    }
} 
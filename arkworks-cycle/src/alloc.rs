use std::borrow::Borrow;

use ark_ff::PrimeField;
use ark_r1cs_std::{prelude::{AllocVar, AllocationMode}, boolean::Boolean, uint8::UInt8};
use ark_relations::r1cs::{Namespace, SynthesisError};

use crate::{Uint8Array, BooleanArray, Boolean2DArray, Boolean3DArray};

impl<const N: usize, F: PrimeField> AllocVar<[u8; N], F> for Uint8Array<N, F> {
    fn new_variable<T: Borrow<[u8; N]>>(
        cs: impl Into<Namespace<F>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let cs = cs.into();
        let mut array = [(); N].map(|_| UInt8::constant(0));
        let value = f().map_or([0; N], |f| *f.borrow());
        for (i, v) in value.into_iter().enumerate() {
            array[i] = UInt8::new_variable(cs.clone(), || Ok(v), mode)?;
        }
        let contraint_array = Uint8Array(array);
        Ok(contraint_array)
    }
} 

impl<const N: usize, F: PrimeField> AllocVar<[bool; N], F> for BooleanArray<N, F> {
    fn new_variable<T: Borrow<[bool; N]>>(
        cs: impl Into<Namespace<F>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let cs = cs.into();
        let mut array = [(); N].map(|_| Boolean::constant(false));
        let value = f().map_or([false; N], |f| *f.borrow());
        for (i, v) in value.into_iter().enumerate() {
            array[i] = Boolean::new_variable(cs.clone(), || Ok(v), mode)?;
        }
        let contraint_array = BooleanArray(array);
        Ok(contraint_array)
    }
} 


impl<const N: usize, F: PrimeField> AllocVar<[[bool; N]; N], F> for Boolean2DArray<N, F> {
    fn new_variable<T: Borrow<[[bool; N]; N]>>(
        cs: impl Into<Namespace<F>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let cs = cs.into();
        let row = [(); N].map(|_| Boolean::constant(false));
        let mut contraint_array = Boolean2DArray([(); N].map(|_| row.clone()));
        let value = f().map_or([[false; N]; N], |f| *f.borrow());
        for (i, row) in value.into_iter().enumerate() {
            for (j, cell) in row.into_iter().enumerate() {
                contraint_array.0[i][j] = Boolean::new_variable(cs.clone(), || Ok(cell), mode)?;
            }
        }
        Ok(contraint_array)
    }
} 


impl<const N: usize, const M: usize, F: PrimeField> AllocVar<[[[bool; N]; N]; M], F> for Boolean3DArray<N, M, F> {
    fn new_variable<T: Borrow<[[[bool; N]; N]; M]>>(
        cs: impl Into<Namespace<F>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let cs = cs.into();
        let row = [(); N].map(|_| Boolean::constant(false));
        let n_square = [(); N].map(|_| row.clone());
        let mut contraint_array = Boolean3DArray([(); M].map(|_| n_square.clone()));
        let value = f().map_or([[[false; N]; N]; M], |f| *f.borrow());
        for (k, nsqaure) in value.into_iter().enumerate() {
            for (i, row) in nsqaure.into_iter().enumerate() {
                for (j, cell) in row.into_iter().enumerate() {
                    contraint_array.0[k][i][j] = Boolean::new_variable(cs.clone(), || Ok(cell), mode)?;
                }
            }
        }
        Ok(contraint_array)
    }
} 
use std::borrow::Borrow;
use ark_ff::PrimeField;
use ark_r1cs_std::{
    prelude::{AllocVar, AllocationMode, Boolean, EqGadget},
    uint8::UInt8,
    R1CSVar, ToBitsGadget,
};
use ark_relations::r1cs::{Namespace, SynthesisError};


pub struct Uint8Array<const N: usize, ConstraintF: PrimeField>(pub [UInt8<ConstraintF>; N]);
pub struct BooleanArray<const N: usize, ConstraintF: PrimeField>(pub [Boolean<ConstraintF>; N]);
pub struct Boolean2DArray<const N: usize, ConstraintF: PrimeField>(pub [[Boolean<ConstraintF>; N]; N]);
pub struct Boolean3DArray<const N: usize, const M: usize, ConstraintF: PrimeField>(
    pub [[[Boolean<ConstraintF>; N]; N]; M],
);

// Allocates memory for Uint8Array in our constrains system
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

impl<const N: usize, const M: usize, F: PrimeField> AllocVar<[[[bool; N]; N]; M], F>
    for Boolean3DArray<N, M, F>
{
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
                    contraint_array.0[k][i][j] =
                        Boolean::new_variable(cs.clone(), || Ok(cell), mode)?;
                }
            }
        }
        Ok(contraint_array)
    }
}

// allows comparision ops for ConstraintF
pub trait CmpGadget<ConstraintF: PrimeField>: R1CSVar<ConstraintF> + EqGadget<ConstraintF> {
    #[inline]
    fn is_geq(&self, other: &Self) -> Result<Boolean<ConstraintF>, SynthesisError> {
        // self >= other => self == other || self > other
        //               => !(self < other)
        self.is_lt(other).map(|b| b.not())
    }

    #[inline]
    fn is_leq(&self, other: &Self) -> Result<Boolean<ConstraintF>, SynthesisError> {
        // self <= other => self == other || self < other
        //               => self == other || other > self
        //               => self >= other
        other.is_geq(self)
    }

    #[inline]
    fn is_gt(&self, other: &Self) -> Result<Boolean<ConstraintF>, SynthesisError> {
        // self > other => !(self == other  || self < other)
        //              => !(self <= other)
        self.is_leq(other).map(|b| b.not())
    }

    fn is_lt(&self, other: &Self) -> Result<Boolean<ConstraintF>, SynthesisError>;
}

impl<ConstraintF: PrimeField> CmpGadget<ConstraintF> for UInt8<ConstraintF> {
    fn is_lt(&self, other: &Self) -> Result<Boolean<ConstraintF>, SynthesisError> {
        // Determine the variable mode.
        if self.is_constant() && other.is_constant() {
            let self_value = self.value().unwrap();
            let other_value = other.value().unwrap();
            let result = Boolean::constant(self_value < other_value);
            Ok(result)
        } else {
            let diff_bits = self.xor(other)?.to_bits_be()?.into_iter();
            let mut result = Boolean::FALSE;
            let mut a_and_b_equal_so_far = Boolean::TRUE;
            let a_bits = self.to_bits_be()?;
            let b_bits = other.to_bits_be()?;
            for ((a_and_b_are_unequal, a), b) in diff_bits.zip(a_bits).zip(b_bits) {
                let a_is_lt_b = a.not().and(&b)?;
                let a_and_b_are_equal = a_and_b_are_unequal.not();
                result = result.or(&a_is_lt_b.and(&a_and_b_equal_so_far)?)?;
                a_and_b_equal_so_far = a_and_b_equal_so_far.and(&a_and_b_are_equal)?;
            }
            Ok(result)
        }
    }
}

#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

use ark_groth16::{Groth16, prepare_verifying_key};
use ark_crypto_primitives::snark::{CircuitSpecificSetupSNARK, SNARK};
use ark_ec::pairing::Pairing;
use ark_ff::Field;
use ark_relations::{
    lc,
    r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError},
};
use ark_std::{
    rand::{RngCore, SeedableRng},
    test_rng, UniformRand,
};
use ark_serialize::{CanonicalSerialize, Write};
use std::fs::File;
use ark_std::error::Error;


fn main() {
    // TODO: add IO?
}

struct MySillyCircuit<F: Field> {
    a: Option<F>,
    b: Option<F>,
}

impl<ConstraintF: Field> ConstraintSynthesizer<ConstraintF> for MySillyCircuit<ConstraintF> {
    fn generate_constraints(
        self,
        cs: ConstraintSystemRef<ConstraintF>,
    ) -> Result<(), SynthesisError> {
        let a = cs.new_witness_variable(|| self.a.ok_or(SynthesisError::AssignmentMissing))?;
        let b = cs.new_witness_variable(|| self.b.ok_or(SynthesisError::AssignmentMissing))?;
        let c = cs.new_input_variable(|| {
            let mut a = self.a.ok_or(SynthesisError::AssignmentMissing)?;
            let b = self.b.ok_or(SynthesisError::AssignmentMissing)?;

            a *= &b;
            Ok(a)
        })?;

        cs.enforce_constraint(lc!() + a, lc!() + b, lc!() + c)?;
        cs.enforce_constraint(lc!() + a, lc!() + b, lc!() + c)?;
        cs.enforce_constraint(lc!() + a, lc!() + b, lc!() + c)?;
        cs.enforce_constraint(lc!() + a, lc!() + b, lc!() + c)?;
        cs.enforce_constraint(lc!() + a, lc!() + b, lc!() + c)?;
        cs.enforce_constraint(lc!() + a, lc!() + b, lc!() + c)?;

        Ok(())
    }
}

fn test_prove_and_verify<E>() -> Result<(), Box<dyn Error>>
where
    E: Pairing,
{
    let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(test_rng().next_u64());

    let (pk, vk) = Groth16::<E>::setup(MySillyCircuit { a: None, b: None }, &mut rng).unwrap();
    let pvk = prepare_verifying_key::<E>(&vk);


    let a = E::ScalarField::rand(&mut rng);
    let b = E::ScalarField::rand(&mut rng);
    let mut c = a;
    c *= b;

    let proof = Groth16::<E>::prove(
        &pk,
        MySillyCircuit {
            a: Some(a),
            b: Some(b),
        },
        &mut rng,
    )
    .unwrap();

    assert!(Groth16::<E>::verify_with_processed_vk(&pvk, &[c], &proof).unwrap());
    assert!(!Groth16::<E>::verify_with_processed_vk(&pvk, &[a], &proof).unwrap());

    let mut compressed_bytes = Vec::new();
    proof.serialize_compressed(&mut compressed_bytes).unwrap();
    let file_path = "./proof";
    let mut file: File = File::create(file_path)?;
    file.write_all(&compressed_bytes)?;
    file.flush()?;
    Ok(())
}

mod bls12_381 {
    use super::{test_prove_and_verify};
    use ark_bls12_381::Bls12_381;

    #[test]
    fn prove_and_verify() {
        let _ = test_prove_and_verify::<Bls12_381>().unwrap();
    }
}

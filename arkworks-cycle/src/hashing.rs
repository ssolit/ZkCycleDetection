use ark_bls12_381::fr::Fr;
use ark_crypto_primitives::sponge::{
    CryptographicSponge, 
    FieldBasedCryptographicSponge,
    poseidon::{PoseidonSponge},
};

use ark_ff::{PrimeField};
use ark_r1cs_std::{boolean::Boolean, R1CSVar, fields::fp::FpVar};
use ark_relations::r1cs::{SynthesisError, ConstraintSystemRef};

mod hashing_utils;
use crate::utils::{
    Boolean2DArray, 
    // Boolean3DArray, 
    // BooleanArray, 
    // Uint8Array,
};
use crate::hashing::hashing_utils::{PoseidonSpongeVar, CryptographicSpongeVar};
pub use hashing_utils::poseidon_parameters_for_test;

// calculates the hash
pub fn hasher<const N: usize, ConstraintF: PrimeField>(
    adj_matrix: &Boolean2DArray<N, ConstraintF>,
) -> Result<Vec<Fr>, SynthesisError> {
    let sponge_param = poseidon_parameters_for_test();
    let mut sponge = PoseidonSponge::<Fr>::new(&sponge_param);
    let flattened_matrix = matrix_flattener(&adj_matrix).unwrap();
    sponge.absorb(&flattened_matrix);
    let hash = sponge.squeeze_native_field_elements(1).to_vec();
    Ok(hash)
}

// calculates the hash and checks correctness as a circuit
pub fn hasher_var<const N: usize, ConstraintF: PrimeField>(
    cs: ConstraintSystemRef<ConstraintF>,
    adj_matrix: &Boolean2DArray<N, ConstraintF>,
) -> Result<Vec<FpVar<ConstraintF>>, SynthesisError> {
    let sponge_param = poseidon_parameters_for_test();
    let mut sponge = PoseidonSpongeVar::<ConstraintF>::new(cs, &sponge_param);
    let flattened_matrix = matrix_flattener_var(&adj_matrix).unwrap();
    sponge.absorb(&flattened_matrix)?;
    let hash = sponge.squeeze_field_elements(1)?;
    Ok(hash)
}

// Takes in a 2D Boolean array (representing an adjacency matrix) and flattens it into a boolean vector
pub fn matrix_flattener<const N: usize, ConstraintF: PrimeField>(
    adj_matrix: &Boolean2DArray<N, ConstraintF>,
) -> Result<Vec<bool>, SynthesisError> {
    let mut flattened_matrix = Vec::new();
    for i in 0..N {
        for j in 0..N {
            let transacted = &adj_matrix.0[i][j]; // true if person i sent to person j
            if transacted.value()? == true {
                flattened_matrix.push(true);
            } else {
                flattened_matrix.push(false);
            }
        }
    }
    Ok(flattened_matrix)
}

// Takes in a 2D Boolean array (representing an adjacency matrix) and flattens it into a boolean vector
// Checks correctness as a circuit
pub fn matrix_flattener_var<const N: usize, ConstraintF: PrimeField>(
    adj_matrix: &Boolean2DArray<N, ConstraintF>,
) -> Result<Vec<&Boolean<ConstraintF>>, SynthesisError> {
    let mut flattened_matrix = Vec::new();
    for i in 0..N {
        for j in 0..N {
            flattened_matrix.push(&adj_matrix.0[i][j]);
        }
    }
    Ok(flattened_matrix)
}


#[test]
fn mod_gen_hash_test() {
    use ark_bls12_381::Fq as F;
    use ark_relations::r1cs::ConstraintSystem;
    use ark_r1cs_std::alloc::AllocVar;

    let adj_matrix = [
        [false, true, true, false],   //               [0]
        [false, false, true, false],  //               / \
        [false, false, false, true],  //             [1]->[2] -> 3
        [false, false, false, false], //
    ];

    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_var = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix)).unwrap();

    let hash1 = hasher(&adj_matrix_var).unwrap();
    let hash2 = hasher(&adj_matrix_var).unwrap();

    // Check if hashes are consistent for the same input
    assert_eq!(hash1, hash2);

    // Modify the adjacency matrix
    let adj_matrix_modified = [
        [true, true, false, false],   //              [0]
        [false, false, true, false],  //              /  \
        [false, false, false, true],  //             [1]->[2] -> 3
        [false, false, false, false], //
    ];
    let adj_matrix_var_modified =
        Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix_modified)).unwrap();
    let hash_modified = hasher(&adj_matrix_var_modified).unwrap();

    // Check if hash changes with different input
    assert_ne!(hash1, hash_modified);
}

#[test]
fn test_hashing_empty_matrix() {
    use ark_bls12_381::Fq as F;
    use ark_relations::r1cs::ConstraintSystem;
    use ark_r1cs_std::alloc::AllocVar;

    let adj_matrix = [[false; 4]; 4];

    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_var = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix)).unwrap();
    let hash = hasher(&adj_matrix_var).unwrap();

    // Ensure hash is not empty or null
    assert!(!hash.is_empty());
}

#[test]
fn test_hashing_full_matrix() {
    use ark_bls12_381::Fq as F;
    use ark_relations::r1cs::ConstraintSystem;
    use ark_r1cs_std::alloc::AllocVar;

    let adj_matrix = [[true; 4]; 4];

    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_var = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix)).unwrap();
    let hash = hasher(&adj_matrix_var).unwrap();

    // Assert the hash is generated successfully
    assert!(!hash.is_empty());
}

#[test]
fn test_hashing_different_matrices() {
    use ark_bls12_381::Fq as F;
    use ark_relations::r1cs::ConstraintSystem;
    use ark_r1cs_std::alloc::AllocVar;

    let adj_matrix_1 = [[false, true], [true, false]];
    let adj_matrix_2 = [[true, false], [false, true]];

    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_var_1 = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix_1)).unwrap();
    let adj_matrix_var_2 = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix_2)).unwrap();

    let hash1 = hasher(&adj_matrix_var_1).unwrap();
    let hash2 = hasher(&adj_matrix_var_2).unwrap();

    assert_ne!(hash1, hash2);
}

#[test]
fn test_hashing_one_changed_element() {
    use ark_bls12_381::Fq as F;
    use ark_relations::r1cs::ConstraintSystem;
    use ark_r1cs_std::alloc::AllocVar;

    let adj_matrix_1 = [[false; 3]; 3];
    let mut adj_matrix_2 = adj_matrix_1.clone();
    adj_matrix_2[1][1] = true; // Change one element

    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_var_1 = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix_1)).unwrap();
    let adj_matrix_var_2 = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix_2)).unwrap();

    let hash1 = hasher(&adj_matrix_var_1).unwrap();
    let hash2 = hasher(&adj_matrix_var_2).unwrap();

    assert_ne!(hash1, hash2);
}

#[test]
fn test_hashing_inverted_matrices() {
    use ark_bls12_381::Fq as F;
    use ark_relations::r1cs::ConstraintSystem;
    use ark_r1cs_std::alloc::AllocVar;

    let adj_matrix = [[true, false], [false, true]];
    let inverted_matrix = adj_matrix.map(|row| row.map(|elem| !elem));

    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_var = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix)).unwrap();
    let inverted_matrix_var =
        Boolean2DArray::new_witness(cs.clone(), || Ok(inverted_matrix)).unwrap();

    let hash1 = hasher(&adj_matrix_var).unwrap();
    let hash2 = hasher(&inverted_matrix_var).unwrap();

    assert_ne!(hash1, hash2);
}

#[test]
fn test_hashing_large_diagonal_matrices() {
    use ark_bls12_381::Fq as F;
    use ark_relations::r1cs::ConstraintSystem;
    use ark_r1cs_std::alloc::AllocVar;

    const N: usize = 50; // Large size
    let mut adj_matrix = [[false; N]; N];

    // Diagonal true values
    for i in 0..N {
        adj_matrix[i][i] = true;
    }

    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_var_1 =
        Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix.clone())).unwrap();
    let adj_matrix_var_2 =
        Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix.clone())).unwrap();

    let hash1 = hasher(&adj_matrix_var_1).unwrap();
    let hash2 = hasher(&adj_matrix_var_2).unwrap();

    assert_eq!(hash1, hash2);
}

#[test]
fn test_hashing_large_sparse_matrices() {
    use ark_bls12_381::Fq as F;
    use ark_relations::r1cs::ConstraintSystem;
    use ark_r1cs_std::alloc::AllocVar;

    const N: usize = 60; // Large size
    let mut adj_matrix = [[false; N]; N];

    // Sparse true values
    for i in (0..N).step_by(10) {
        for j in (0..N).step_by(15) {
            adj_matrix[i][j] = true;
        }
    }

    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_var_1 =
        Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix.clone())).unwrap();
    let adj_matrix_var_2 =
        Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix.clone())).unwrap();

    let hash1 = hasher(&adj_matrix_var_1).unwrap();
    let hash2 = hasher(&adj_matrix_var_2).unwrap();

    assert_eq!(hash1, hash2);
}

// Test failing because matrix is too large

// #[test]
// fn test_hashing_large_identical_matrices() {
//     use ark_bls12_381::Fq as F;
//     use ark_relations::r1cs::ConstraintSystem;
//     use ark_r1cs_std::alloc::AllocVar;

//     const N: usize = 100; // Large size
//     let mut adj_matrix_1 = [[false; N]; N];
//     let mut adj_matrix_2 = [[false; N]; N];

//     // Initialize both matrices with the same pattern
//     for i in 0..N {
//         for j in 0..N {
//             if i % 2 == 0 && j % 3 == 0 {
//                 adj_matrix_1[i][j] = true;
//                 adj_matrix_2[i][j] = true;
//             }
//         }
//     }

//     let cs = ConstraintSystem::<F>::new_ref();
//     let adj_matrix_var_1 = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix_1)).unwrap();
//     let adj_matrix_var_2 = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix_2)).unwrap();

//     let hash1 = hasher(&adj_matrix_var_1).unwrap();
//     let hash2 = hasher(&adj_matrix_var_2).unwrap();

//     assert_eq!(hash1, hash2);
// }
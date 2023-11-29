#![allow(unused_imports)]
#![allow(dead_code)]

use ark_ff::PrimeField;
use ark_r1cs_std::{
    prelude::{Boolean, EqGadget, AllocVar},
    uint8::UInt8
};
use ark_relations::r1cs::{SynthesisError, ConstraintSystem};
use tracing_subscriber::layer::SubscriberExt;
use cmp::CmpGadget;

mod cmp;
mod alloc;

pub struct Boolean2DArray<const N: usize, ConstraintF: PrimeField>([[Boolean<ConstraintF>; N]; N]);
pub struct Uint8Array<const N: usize, ConstraintF: PrimeField>([UInt8<ConstraintF>; N]);
pub struct BooleanArray<const N: usize, ConstraintF: PrimeField>([Boolean<ConstraintF>; N]);


// special case where every node should be considered
fn check_topo_sort<const N: usize, ConstraintF: PrimeField>(
    adj_matrix: &Boolean2DArray<N, ConstraintF>, 
    topo: &Uint8Array<N, ConstraintF>,
) -> Result<(), SynthesisError> {
    let subgraph_nodes = &BooleanArray([(); N].map(|_| Boolean::constant(true)));
    check_subgraph_topo_sort(adj_matrix, subgraph_nodes, topo)
}

// Challenge: can't leak the size of the subgraph
// NOTE: probably need to do more to check a toposort is valid
// ex no duplictes nodes listed, rn can list same node N times. 
fn check_subgraph_topo_sort<const N: usize, ConstraintF: PrimeField>(
    adj_matrix: &Boolean2DArray<N, ConstraintF>, 
    subgraph_nodes: &BooleanArray<N, ConstraintF>,
    topo: &Uint8Array<N, ConstraintF>,
) -> Result<(), SynthesisError> {
    for i in 0..N {
        for j in i+1..N {
            let transacted = &adj_matrix.0[i][j]; // true if person i sent to person j
            let sender_in_subgraph = &subgraph_nodes.0[i];
            let reciever_in_subgraph = &subgraph_nodes.0[j];

            // The subgraph should represent all nodes reachable from a start node
            // The subgraph is bad if the transaction goes from within subgraph to outside it
            let bad_subgraph = transacted
                                    .and(sender_in_subgraph)?
                                    .and(&reciever_in_subgraph.not())?
                                    .and(transacted)?;
            let _ = bad_subgraph.enforce_equal(&Boolean::FALSE);       


            let wrong_order = topo.0[i].is_gt(&topo.0[j])?; // i is later in the topo sort than j 
            let bad_subgraph_toposort = transacted
                                            .and(sender_in_subgraph)?
                                            .and(reciever_in_subgraph)?
                                            .and(&wrong_order)?;
            let _ = bad_subgraph_toposort.enforce_equal(&Boolean::FALSE);
        }
    }
    Ok(())
}

fn check_helper<const N: usize, ConstraintF: PrimeField>(
    adj_matrix: &[[bool; N]; N],
    topo: &[u8; N],
) {
    let cs = ConstraintSystem::<ConstraintF>::new_ref();
    let adj_matrix_var = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix)).unwrap();
    let topo_var = Uint8Array::new_witness(cs.clone(), || Ok(topo)).unwrap();
    check_topo_sort(&adj_matrix_var, &topo_var).unwrap();
    // //TODO: check hash of adj_matrix matches some public input
    assert!(cs.is_satisfied().unwrap());
}

fn main() {
    // TODO: add IO?
}

#[test]
fn valid_topo_sort() {
    use ark_relations::r1cs::{ConstraintLayer, ConstraintSystem, TracingMode};
    use tracing_subscriber::layer::SubscriberExt;
    use ark_bls12_381::Fq as F;

    // supposed to give debug traces, but didn't
    let mut layer = ConstraintLayer::default();
    layer.mode = TracingMode::OnlyConstraints;
    let subscriber = tracing_subscriber::Registry::default().with(layer);
    let _guard = tracing::subscriber::set_default(subscriber);

    // Check that it accepts a valid solution.
    let adj_matrix = [                                      
        [false, true, true, false],  //               [0]
        [false, false, true, false], //               / \
        [false, false, false, true], //             [1]->[2] -> 3
        [false, false, false, false] //                     
    ];
    let topo = [0, 1, 2, 3];
    

    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_var = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix)).unwrap();
    let topo_var = Uint8Array::new_witness(cs.clone(), || Ok(topo)).unwrap();
    check_topo_sort(&adj_matrix_var, &topo_var).unwrap();
    // //TODO: check hash of adj_matrix matches some public input
    let is_satisfied = cs.is_satisfied().unwrap();
    if !is_satisfied {
        // If it isn't, find out the offending constraint.
        println!("{:?}", cs.which_is_unsatisfied());
    }
    assert!(is_satisfied);
}

#[test]
fn invalid_topo_sort() {
    use ark_relations::r1cs::{ConstraintLayer, ConstraintSystem, TracingMode};
    use tracing_subscriber::layer::SubscriberExt;
    use ark_bls12_381::Fq as F;

    // supposed to give debug traces, but didn't
    let mut layer = ConstraintLayer::default();
    layer.mode = TracingMode::OnlyConstraints;
    let subscriber = tracing_subscriber::Registry::default().with(layer);
    let _guard = tracing::subscriber::set_default(subscriber);

    // Check that it accepts a valid solution.
    let adj_matrix = [                                      
        [false, true, true, false],  //               [0]
        [false, false, true, false], //               / \
        [false, false, false, true], //             [1]->[2] -> 3
        [false, false, false, false] //                     
    ];
    let topo = [1, 0, 2, 3]; // bad because 0->1
    

    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_var = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix)).unwrap();
    let topo_var = Uint8Array::new_witness(cs.clone(), || Ok(topo)).unwrap();
    check_topo_sort(&adj_matrix_var, &topo_var).unwrap();
    // //TODO: check hash of adj_matrix matches some public input
    let is_satisfied = cs.is_satisfied().unwrap();
    if !is_satisfied {
        // If it isn't, find out the offending constraint.
        println!("{:?}", cs.which_is_unsatisfied());
    }
    assert!(!is_satisfied);
}

#[test]
fn topo_sort_missing_nodes() {
    use ark_relations::r1cs::{ConstraintLayer, ConstraintSystem, TracingMode};
    use tracing_subscriber::layer::SubscriberExt;
    use ark_bls12_381::Fq as F;

    // supposed to give debug traces, but didn't
    let mut layer = ConstraintLayer::default();
    layer.mode = TracingMode::OnlyConstraints;
    let subscriber = tracing_subscriber::Registry::default().with(layer);
    let _guard = tracing::subscriber::set_default(subscriber);

    // Check that it accepts a valid solution.
    let adj_matrix = [                                      
        [false, true, true, false],  //               [0]
        [false, false, true, false], //               / \
        [false, false, false, true], //             [1]->[2] -> 3
        [false, false, false, false] //                     
    ];
    let topo = [0, 0, 0, 0];
    

    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_var = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix)).unwrap();
    let topo_var = Uint8Array::new_witness(cs.clone(), || Ok(topo)).unwrap();
    check_topo_sort(&adj_matrix_var, &topo_var).unwrap();
    // //TODO: check hash of adj_matrix matches some public input
    let is_satisfied = cs.is_satisfied().unwrap();
    if !is_satisfied {
        // If it isn't, find out the offending constraint.
        println!("{:?}", cs.which_is_unsatisfied());
    }
    assert!(!is_satisfied);
}

#[test]
fn valid_subgraph_sort() {
    use ark_relations::r1cs::{ConstraintLayer, ConstraintSystem, TracingMode};
    use tracing_subscriber::layer::SubscriberExt;
    use ark_bls12_381::Fq as F;

    // supposed to give debug traces, but didn't
    let mut layer = ConstraintLayer::default();
    layer.mode = TracingMode::OnlyConstraints;
    let subscriber = tracing_subscriber::Registry::default().with(layer);
    let _guard = tracing::subscriber::set_default(subscriber);

    // Check that it accepts a valid solution.
    let adj_matrix = [                                      
        [false, true, true, false],  //               [0]
        [false, false, true, false], //               / \
        [false, false, false, true], //             [1]->[2] -> 3
        [false, false, false, false] //                     
    ];
    let subgraph_nodes = [false, true, true, true]; // simulate node 1's subgraph, 0 isn't reachable so ignore 
    let topo = [1, 0, 2, 3]; 
    

    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_var = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix)).unwrap();
    let subgraph_nodes_var = BooleanArray::new_witness(cs.clone(), || Ok(subgraph_nodes)).unwrap();
    let topo_var = Uint8Array::new_witness(cs.clone(), || Ok(topo)).unwrap();
    check_subgraph_topo_sort(&adj_matrix_var, &subgraph_nodes_var, &topo_var).unwrap();
    // //TODO: check hash of adj_matrix matches some public input
    let is_satisfied = cs.is_satisfied().unwrap();
    if !is_satisfied {
        // If it isn't, find out the offending constraint.
        println!("offending constaint");
        println!("{:?}", cs.which_is_unsatisfied());
    }
    assert!(is_satisfied);
}

#[test]
fn invalid_subgraph_topo() {
    use ark_relations::r1cs::{ConstraintLayer, ConstraintSystem, TracingMode};
    use tracing_subscriber::layer::SubscriberExt;
    use ark_bls12_381::Fq as F;

    // supposed to give debug traces, but didn't
    let mut layer = ConstraintLayer::default();
    layer.mode = TracingMode::OnlyConstraints;
    let subscriber = tracing_subscriber::Registry::default().with(layer);
    let _guard = tracing::subscriber::set_default(subscriber);

    let adj_matrix = [                                      
        [false, true, true, false],  //               [0]
        [false, false, true, false], //               / \
        [false, false, false, true], //             [1]->[2] -> 3
        [false, false, false, false] //                     
    ];
    let subgraph_nodes = [false, true, true, true]; // invalid because 2 is included and 3 is not, yet 2 -> 3
    let topo = [0, 2, 1, 3]; 
    

    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_var = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix)).unwrap();
    let subgraph_nodes_var = BooleanArray::new_witness(cs.clone(), || Ok(subgraph_nodes)).unwrap();
    let topo_var = Uint8Array::new_witness(cs.clone(), || Ok(topo)).unwrap();
    check_subgraph_topo_sort(&adj_matrix_var, &subgraph_nodes_var, &topo_var).unwrap();
    // //TODO: check hash of adj_matrix matches some public input
    let is_satisfied = cs.is_satisfied().unwrap();
    if !is_satisfied {
        // If it isn't, find out the offending constraint.
        println!("offending constaint");
        println!("{:?}", cs.which_is_unsatisfied());
    }
    assert!(!is_satisfied);
}
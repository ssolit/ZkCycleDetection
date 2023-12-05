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
fn invalid_topo_sort_2() {
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
        [false, true, true, false, false, false],  //               [0]<-----\
        [false, false, true, false, false, false], //               / \       \ 
        [false, false, false, true, false, true], //             [1]->[2]      \
        [false, false, false, false, false, false], //                /  \     /
        [true, false, false, false, false, false], //               [3]  [5]->[4]
        [false, false, false, false, true, false], //                     
    ];
    let topo = [0, 1, 2, 3, 4, 5]; // bad because 4 -> 0
    

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
    let topo = [0, 0, 0, 0]; // bad because not including all nodes
    

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
    let topo = [1, 0, 2, 3]; // 0 is ignored, so order its spot in the sort doesn't matter
    

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
fn valid_subgraph_sort_ignores_cycle() {
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
        [false, true, true, false, false, false],  //               [0]          
        [false, false, true, false, false, false], //               / \
        [false, false, false, true, false, false], //             [1]->[2]->[3]          
        [false, false, false, false, false, false], //
        [false, false, false, false, false, true], //               [4]<->[5]
        [false, false, false, false, true, false], //                     
    ];
    let subgraph_nodes = [true, true, true, true, false, false]; // node 4+5 are ignored
    let topo = [0, 1, 2, 3, 4, 5]; 
    

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

#[test]
fn valid_multi() {
    use ark_relations::r1cs::{ConstraintLayer, ConstraintSystem, TracingMode};
    use tracing_subscriber::layer::SubscriberExt;
    use ark_bls12_381::Fq as F;

    // supposed to give debug traces, but didn't
    let mut layer = ConstraintLayer::default();
    layer.mode = TracingMode::OnlyConstraints;
    let subscriber = tracing_subscriber::Registry::default().with(layer);
    let _guard = tracing::subscriber::set_default(subscriber);

    let adj_matrix_1 = [                                      
        [false, true, true, false, false, false],  //               [0]
        [false, false, true, false, false, false], //               / \
        [false, false, false, false, false, false], //             [1]->[2]
        [false, false, false, false, false, false], //
        [false, false, false, false, false, false], // 
        [false, false, false, false, false, false], //                     
    ];
    let adj_matrix_2 = [                                      
        [false, false, false, false, false, false], //               
        [false, false, false, false, false, false], //               
        [false, false, false, true, false, false], //             [2] -> [3] -> [5]
        [false, false, false, false, false, true], //
        [false, false, false, false, false, false], // 
        [false, false, false, false, false, false], //                     
    ];

    let adj_matrix_3 = [                                      
        [false, false, false, false, false, false], //               
        [false, false, false, false, false, false], //              
        [false, false, false, false, false, false], //             
        [false, false, false, false, false, false], //          [5] -> [4]
        [false, false, false, false, false, false], // 
        [false, false, false, false, true, false], //                     
    ];

    let adj_matrix_array = [adj_matrix_1, adj_matrix_2, adj_matrix_3];
    let subgraph_nodes = [true, true, true, true, true, true]; 
    let topo = [0, 1, 2, 3, 5, 4]; 


    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_array_var = Boolean3DArray::new_witness(cs.clone(), || Ok(adj_matrix_array)).unwrap();
    let subgraph_nodes_var = BooleanArray::new_witness(cs.clone(), || Ok(subgraph_nodes)).unwrap();
    let topo_var = Uint8Array::new_witness(cs.clone(), || Ok(topo)).unwrap();
    check_multi_subgraph_topo_sort(&adj_matrix_array_var, &subgraph_nodes_var, &topo_var).unwrap();
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
fn invalid_multi_bad_topo() {
    use ark_relations::r1cs::{ConstraintLayer, ConstraintSystem, TracingMode};
    use tracing_subscriber::layer::SubscriberExt;
    use ark_bls12_381::Fq as F;

    // supposed to give debug traces, but didn't
    let mut layer = ConstraintLayer::default();
    layer.mode = TracingMode::OnlyConstraints;
    let subscriber = tracing_subscriber::Registry::default().with(layer);
    let _guard = tracing::subscriber::set_default(subscriber);

    let adj_matrix_1 = [                                      
        [false, true, true, false, false, false],  //               [0]
        [false, false, true, false, false, false], //               / \
        [false, false, false, false, false, false], //             [1]->[2]
        [false, false, false, false, false, false], //
        [false, false, false, false, false, false], // 
        [false, false, false, false, false, false], //                     
    ];
    let adj_matrix_2 = [                                      
        [false, false, false, false, false, false], //               
        [false, false, false, false, false, false], //               
        [false, false, false, true, false, false], //             [2] -> [3] -> [5]
        [false, false, false, false, false, true], //
        [false, false, false, false, false, false], // 
        [false, false, false, false, false, false], //                     
    ];

    let adj_matrix_3 = [                                      
        [false, false, false, false, false, false], //              [5]
        [false, false, true, false, false, false], //               / \
        [false, false, false, false, false, false], //             [0] [4]         
        [false, false, false, false, false, false], //          
        [false, false, false, false, false, false], // 
        [true, false, false, false, true, false], //                     
    ];

    let adj_matrix_array = [adj_matrix_1, adj_matrix_2, adj_matrix_3];
    let subgraph_nodes = [true, true, true, true, true, true]; 
    let topo = [0, 1, 2, 3, 4, 5]; // bad b/c 5 -> 0


    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_array_var = Boolean3DArray::new_witness(cs.clone(), || Ok(adj_matrix_array)).unwrap();
    let subgraph_nodes_var = BooleanArray::new_witness(cs.clone(), || Ok(subgraph_nodes)).unwrap();
    let topo_var = Uint8Array::new_witness(cs.clone(), || Ok(topo)).unwrap();
    check_multi_subgraph_topo_sort(&adj_matrix_array_var, &subgraph_nodes_var, &topo_var).unwrap();
    // //TODO: check hash of adj_matrix matches some public input
    let is_satisfied = cs.is_satisfied().unwrap();
    if !is_satisfied {
        // If it isn't, find out the offending constraint.
        println!("offending constaint");
        println!("{:?}", cs.which_is_unsatisfied());
    }
    assert!(!is_satisfied);
}
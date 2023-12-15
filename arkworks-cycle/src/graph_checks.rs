use crate::utils::CmpGadget;
use ark_ff::PrimeField;
use ark_r1cs_std::prelude::{Boolean, EqGadget};
use ark_relations::r1cs::SynthesisError;
use crate::utils::{Boolean2DArray, Boolean3DArray, BooleanArray, Uint8Array};

// special case where every node should be considered
pub fn check_topo_sort<const N: usize, ConstraintF: PrimeField>(
    adj_matrix: &Boolean2DArray<N, ConstraintF>,
    topo: &Uint8Array<N, ConstraintF>,
) -> Result<(), SynthesisError> {
    let subgraph_nodes = &BooleanArray([(); N].map(|_| Boolean::constant(true)));
    check_subgraph_topo_sort(adj_matrix, subgraph_nodes, topo)
}

// Challenge: can't leak the size of the subgraph
// NOTE: probably need to do more to check a toposort is valid
// ex no duplictes nodes listed, rn can list same node N times.
pub fn check_subgraph_topo_sort<const N: usize, ConstraintF: PrimeField>(
    adj_matrix: &Boolean2DArray<N, ConstraintF>,
    subgraph_nodes: &BooleanArray<N, ConstraintF>,
    topo: &Uint8Array<N, ConstraintF>,
) -> Result<(), SynthesisError> {
    // check that there are no duplicate numbers in the toposort
    for i in 0..N {
        for j in i + 1..N {
            let gt = &topo.0[i].is_gt(&topo.0[j])?;
            let lt = &topo.0[i].is_lt(&topo.0[j])?;
            let _ = gt.or(lt)?.enforce_equal(&Boolean::TRUE);
        }
    }

    // do checks relating to individual edges
    for i in 0..N {
        for j in 0..N {
            let transacted = &adj_matrix.0[i][j]; // true if person i sent to person j
            let sender_in_subgraph = &subgraph_nodes.0[i];
            let reciever_in_subgraph = &subgraph_nodes.0[j];

            // Check no edges going out of the subgraph
            // Which is claimed to be every node reachable from some start node
            let bad_subgraph = transacted
                .and(sender_in_subgraph)?
                .and(&reciever_in_subgraph.not())?;
            let _ = bad_subgraph.enforce_equal(&Boolean::FALSE);

            // check if toposort is invalid because of a backwards edge
            let wrong_order = topo.0[i].is_gt(&topo.0[j])?; // i is later in the topo sort than j
            let backwards_edge = transacted
                .and(sender_in_subgraph)?
                .and(reciever_in_subgraph)?
                .and(&wrong_order)?;
            let _ = backwards_edge.enforce_equal(&Boolean::FALSE);
        }
    }
    Ok(())
}

// Combines the adj matricies into one matrix
pub fn check_multi_subgraph_topo_sort<const N: usize, const M: usize, ConstraintF: PrimeField>(
    adj_matrix_array: &Boolean3DArray<N, M, ConstraintF>,
    subgraph_nodes: &BooleanArray<N, ConstraintF>,
    topo: &Uint8Array<N, ConstraintF>,
) -> Result<(), SynthesisError> {
    let combined_adj_matrix = &mut Boolean2DArray(adj_matrix_array.0[0].clone());

    for k in 1..M {
        for i in 0..N {
            for j in 0..N {
                combined_adj_matrix.0[i][j] =
                    combined_adj_matrix.0[i][j].or(&adj_matrix_array.0[k][i][j])?;
            }
        }
    }
    check_subgraph_topo_sort(combined_adj_matrix, subgraph_nodes, topo)
}

#[test]
fn valid_topo_sort() {
    use ark_bls12_381::Fq as F;
    use ark_relations::r1cs::{ConstraintLayer, ConstraintSystem, TracingMode};
    use tracing_subscriber::layer::SubscriberExt;
    use ark_r1cs_std::alloc::AllocVar;

    // supposed to give debug traces, but didn't
    let mut layer = ConstraintLayer::default();
    layer.mode = TracingMode::OnlyConstraints;
    let subscriber = tracing_subscriber::Registry::default().with(layer);
    let _guard = tracing::subscriber::set_default(subscriber);

    // Check that it accepts a valid solution.
    let adj_matrix = [
        [false, true, true, false],   //               [0]
        [false, false, true, false],  //               / \
        [false, false, false, true],  //             [1]->[2] -> 3
        [false, false, false, false], //
    ];
    let topo = [0, 1, 2, 3];

    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_var = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix)).unwrap();
    let topo_var = Uint8Array::new_witness(cs.clone(), || Ok(topo)).unwrap();
    check_topo_sort(&adj_matrix_var, &topo_var).unwrap();
    let is_satisfied = cs.is_satisfied().unwrap();
    if !is_satisfied {
        // If it isn't, find out the offending constraint.
        println!("{:?}", cs.which_is_unsatisfied());
    }
    assert!(is_satisfied);
}

#[test]
fn invalid_topo_sort() {
    use ark_bls12_381::Fq as F;
    use ark_relations::r1cs::{ConstraintLayer, ConstraintSystem, TracingMode};
    use tracing_subscriber::layer::SubscriberExt;
    use ark_r1cs_std::alloc::AllocVar;

    // supposed to give debug traces, but didn't
    let mut layer = ConstraintLayer::default();
    layer.mode = TracingMode::OnlyConstraints;
    let subscriber = tracing_subscriber::Registry::default().with(layer);
    let _guard = tracing::subscriber::set_default(subscriber);

    // Check that it accepts a valid solution.
    let adj_matrix = [
        [false, true, true, false],   //               [0]
        [false, false, true, false],  //               / \
        [false, false, false, true],  //             [1]->[2] -> 3
        [false, false, false, false], //
    ];
    let topo = [1, 0, 2, 3]; // bad because 0->1

    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_var = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix)).unwrap();
    let topo_var = Uint8Array::new_witness(cs.clone(), || Ok(topo)).unwrap();
    check_topo_sort(&adj_matrix_var, &topo_var).unwrap();
    let is_satisfied = cs.is_satisfied().unwrap();
    if !is_satisfied {
        // If it isn't, find out the offending constraint.
        println!("{:?}", cs.which_is_unsatisfied());
    }
    assert!(!is_satisfied);
}

#[test]
fn invalid_topo_sort_2() {
    use ark_bls12_381::Fq as F;
    use ark_relations::r1cs::{ConstraintLayer, ConstraintSystem, TracingMode};
    use tracing_subscriber::layer::SubscriberExt;
    use ark_r1cs_std::alloc::AllocVar;

    // supposed to give debug traces, but didn't
    let mut layer = ConstraintLayer::default();
    layer.mode = TracingMode::OnlyConstraints;
    let subscriber = tracing_subscriber::Registry::default().with(layer);
    let _guard = tracing::subscriber::set_default(subscriber);

    // Check that it accepts a valid solution.
    let adj_matrix = [
        [false, true, true, false, false, false], //               [0]<-----\
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
    let is_satisfied = cs.is_satisfied().unwrap();
    if !is_satisfied {
        // If it isn't, find out the offending constraint.
        println!("{:?}", cs.which_is_unsatisfied());
    }
    assert!(!is_satisfied);
}

#[test]
fn topo_sort_missing_nodes() {
    use ark_bls12_381::Fq as F;
    use ark_relations::r1cs::{ConstraintLayer, ConstraintSystem, TracingMode};
    use tracing_subscriber::layer::SubscriberExt;
    use ark_r1cs_std::alloc::AllocVar;

    // supposed to give debug traces, but didn't
    let mut layer = ConstraintLayer::default();
    layer.mode = TracingMode::OnlyConstraints;
    let subscriber = tracing_subscriber::Registry::default().with(layer);
    let _guard = tracing::subscriber::set_default(subscriber);

    // Check that it accepts a valid solution.
    let adj_matrix = [
        [false, true, true, false],   //               [0]
        [false, false, true, false],  //               / \
        [false, false, false, true],  //             [1]->[2] -> 3
        [false, false, false, false], //
    ];
    let topo = [0, 0, 0, 0]; // bad because not including all nodes

    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_var = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix)).unwrap();
    let topo_var = Uint8Array::new_witness(cs.clone(), || Ok(topo)).unwrap();
    check_topo_sort(&adj_matrix_var, &topo_var).unwrap();
    let is_satisfied = cs.is_satisfied().unwrap();
    if !is_satisfied {
        // If it isn't, find out the offending constraint.
        println!("{:?}", cs.which_is_unsatisfied());
    }
    assert!(!is_satisfied);
}

#[test]
fn valid_subgraph_sort() {
    use ark_bls12_381::Fq as F;
    use ark_relations::r1cs::{ConstraintLayer, ConstraintSystem, TracingMode};
    use tracing_subscriber::layer::SubscriberExt;
    use ark_r1cs_std::alloc::AllocVar;

    // supposed to give debug traces, but didn't
    let mut layer = ConstraintLayer::default();
    layer.mode = TracingMode::OnlyConstraints;
    let subscriber = tracing_subscriber::Registry::default().with(layer);
    let _guard = tracing::subscriber::set_default(subscriber);

    // Check that it accepts a valid solution.
    let adj_matrix = [
        [false, true, true, false],   //               [0]
        [false, false, true, false],  //               / \
        [false, false, false, true],  //             [1]->[2] -> 3
        [false, false, false, false], //
    ];
    let subgraph_nodes = [false, true, true, true]; // simulate node 1's subgraph, 0 isn't reachable so ignore
    let topo = [1, 0, 2, 3]; // 0 is ignored, so order its spot in the sort doesn't matter

    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_var = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix)).unwrap();
    let subgraph_nodes_var = BooleanArray::new_witness(cs.clone(), || Ok(subgraph_nodes)).unwrap();
    let topo_var = Uint8Array::new_witness(cs.clone(), || Ok(topo)).unwrap();
    check_subgraph_topo_sort(&adj_matrix_var, &subgraph_nodes_var, &topo_var).unwrap();
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
    use ark_bls12_381::Fq as F;
    use ark_relations::r1cs::{ConstraintLayer, ConstraintSystem, TracingMode};
    use tracing_subscriber::layer::SubscriberExt;
    use ark_r1cs_std::alloc::AllocVar;

    // supposed to give debug traces, but didn't
    let mut layer = ConstraintLayer::default();
    layer.mode = TracingMode::OnlyConstraints;
    let subscriber = tracing_subscriber::Registry::default().with(layer);
    let _guard = tracing::subscriber::set_default(subscriber);

    // Check that it accepts a valid solution.
    let adj_matrix = [
        [false, true, true, false, false, false], //               [0]
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
    use ark_bls12_381::Fq as F;
    use ark_relations::r1cs::{ConstraintLayer, ConstraintSystem, TracingMode};
    use tracing_subscriber::layer::SubscriberExt;
    use ark_r1cs_std::alloc::AllocVar;

    // supposed to give debug traces, but didn't
    let mut layer = ConstraintLayer::default();
    layer.mode = TracingMode::OnlyConstraints;
    let subscriber = tracing_subscriber::Registry::default().with(layer);
    let _guard = tracing::subscriber::set_default(subscriber);

    let adj_matrix = [
        [false, true, true, false],   //               [0]
        [false, false, true, false],  //               / \
        [false, false, false, true],  //             [1]->[2] -> 3
        [false, false, false, false], //
    ];
    let subgraph_nodes = [false, true, true, true]; // invalid because 2 is included and 3 is not, yet 2 -> 3
    let topo = [0, 2, 1, 3];

    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_var = Boolean2DArray::new_witness(cs.clone(), || Ok(adj_matrix)).unwrap();
    let subgraph_nodes_var = BooleanArray::new_witness(cs.clone(), || Ok(subgraph_nodes)).unwrap();
    let topo_var = Uint8Array::new_witness(cs.clone(), || Ok(topo)).unwrap();
    check_subgraph_topo_sort(&adj_matrix_var, &subgraph_nodes_var, &topo_var).unwrap();
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
    use ark_bls12_381::Fq as F;
    use ark_relations::r1cs::{ConstraintLayer, ConstraintSystem, TracingMode};
    use tracing_subscriber::layer::SubscriberExt;
    use ark_r1cs_std::alloc::AllocVar;

    // supposed to give debug traces, but didn't
    let mut layer = ConstraintLayer::default();
    layer.mode = TracingMode::OnlyConstraints;
    let subscriber = tracing_subscriber::Registry::default().with(layer);
    let _guard = tracing::subscriber::set_default(subscriber);

    let adj_matrix_1 = [
        [false, true, true, false, false, false], //               [0]
        [false, false, true, false, false, false], //               / \
        [false, false, false, false, false, false], //             [1]->[2]
        [false, false, false, false, false, false], //
        [false, false, false, false, false, false], //
        [false, false, false, false, false, false], //
    ];
    let adj_matrix_2 = [
        [false, false, false, false, false, false], //
        [false, false, false, false, false, false], //
        [false, false, false, true, false, false],  //             [2] -> [3] -> [5]
        [false, false, false, false, false, true],  //
        [false, false, false, false, false, false], //
        [false, false, false, false, false, false], //
    ];

    let adj_matrix_3 = [
        [false, false, false, false, false, false], //
        [false, false, false, false, false, false], //
        [false, false, false, false, false, false], //
        [false, false, false, false, false, false], //          [5] -> [4]
        [false, false, false, false, false, false], //
        [false, false, false, false, true, false],  //
    ];

    let adj_matrix_array = [adj_matrix_1, adj_matrix_2, adj_matrix_3];
    let subgraph_nodes = [true, true, true, true, true, true];
    let topo = [0, 1, 2, 3, 5, 4];

    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_array_var =
        Boolean3DArray::new_witness(cs.clone(), || Ok(adj_matrix_array)).unwrap();
    let subgraph_nodes_var = BooleanArray::new_witness(cs.clone(), || Ok(subgraph_nodes)).unwrap();
    let topo_var = Uint8Array::new_witness(cs.clone(), || Ok(topo)).unwrap();
    check_multi_subgraph_topo_sort(&adj_matrix_array_var, &subgraph_nodes_var, &topo_var).unwrap();
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
    use ark_bls12_381::Fq as F;
    use ark_relations::r1cs::{ConstraintLayer, ConstraintSystem, TracingMode};
    use tracing_subscriber::layer::SubscriberExt;
    use ark_r1cs_std::alloc::AllocVar;

    // supposed to give debug traces, but didn't
    let mut layer = ConstraintLayer::default();
    layer.mode = TracingMode::OnlyConstraints;
    let subscriber = tracing_subscriber::Registry::default().with(layer);
    let _guard = tracing::subscriber::set_default(subscriber);

    let adj_matrix_1 = [
        [false, true, true, false, false, false], //               [0]
        [false, false, true, false, false, false], //               / \
        [false, false, false, false, false, false], //             [1]->[2]
        [false, false, false, false, false, false], //
        [false, false, false, false, false, false], //
        [false, false, false, false, false, false], //
    ];
    let adj_matrix_2 = [
        [false, false, false, false, false, false], //
        [false, false, false, false, false, false], //
        [false, false, false, true, false, false],  //             [2] -> [3] -> [5]
        [false, false, false, false, false, true],  //
        [false, false, false, false, false, false], //
        [false, false, false, false, false, false], //
    ];

    let adj_matrix_3 = [
        [false, false, false, false, false, false], //              [5]
        [false, false, true, false, false, false],  //               / \
        [false, false, false, false, false, false], //             [0] [4]
        [false, false, false, false, false, false], //
        [false, false, false, false, false, false], //
        [true, false, false, false, true, false],   //
    ];

    let adj_matrix_array = [adj_matrix_1, adj_matrix_2, adj_matrix_3];
    let subgraph_nodes = [true, true, true, true, true, true];
    let topo = [0, 1, 2, 3, 4, 5]; // bad b/c 5 -> 0

    let cs = ConstraintSystem::<F>::new_ref();
    let adj_matrix_array_var =
        Boolean3DArray::new_witness(cs.clone(), || Ok(adj_matrix_array)).unwrap();
    let subgraph_nodes_var = BooleanArray::new_witness(cs.clone(), || Ok(subgraph_nodes)).unwrap();
    let topo_var = Uint8Array::new_witness(cs.clone(), || Ok(topo)).unwrap();
    check_multi_subgraph_topo_sort(&adj_matrix_array_var, &subgraph_nodes_var, &topo_var).unwrap();
    let is_satisfied = cs.is_satisfied().unwrap();
    if !is_satisfied {
        // If it isn't, find out the offending constraint.
        println!("offending constaint");
        println!("{:?}", cs.which_is_unsatisfied());
    }
    assert!(!is_satisfied);
}

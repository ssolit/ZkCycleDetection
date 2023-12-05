#![allow(unused_imports)]
#![allow(dead_code)]

pub mod graph_checks {

    use crate::{Uint8Array, BooleanArray, Boolean2DArray, Boolean3DArray};
    use ark_ff::PrimeField;
    use ark_r1cs_std::{
        prelude::{Boolean, EqGadget, AllocVar},
        uint8::UInt8
    };
    use ark_relations::r1cs::{SynthesisError, ConstraintSystem};
    use tracing_subscriber::layer::SubscriberExt;
    use crate::cmp::CmpGadget;



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
            for j in i+1..N {
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

                // output starting node for proof?
            }
        }
        Ok(())
    }

    pub fn check_multi_subgraph_topo_sort<const N: usize, const M: usize, ConstraintF: PrimeField>(
        adj_matrix_array: &Boolean3DArray<N, M, ConstraintF>, 
        subgraph_nodes: &BooleanArray<N, ConstraintF>,
        topo: &Uint8Array<N, ConstraintF>,
    ) -> Result<(), SynthesisError> {
        let combined_adj_matrix = &mut Boolean2DArray(adj_matrix_array.0[0].clone());

        for k in 1..M {
            for i in 0..N {
                for j in 0..N {
                    combined_adj_matrix.0[i][j] = combined_adj_matrix.0[i][j].or(&adj_matrix_array.0[k][i][j])?;
                }
            }
        }

        check_subgraph_topo_sort(combined_adj_matrix, subgraph_nodes, topo)
    }
}

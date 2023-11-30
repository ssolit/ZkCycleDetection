use anyhow::{anyhow, Result};
use ark_ed_on_bls12_377::Fq;
use ark_sponge::poseidon::PoseidonSponge;
use ark_sponge::{CryptographicSponge, FieldBasedCryptographicSponge};
mod helper;
type PoseidonHash = PoseidonSponge<Fq>;
pub fn poseidon2_hash(input: &Vec<bool>) -> Result<Fq> {
    let sponge_params = helper::poseidon_parameters_for_test()?;

    let mut native_sponge = PoseidonHash::new(&sponge_params);

    native_sponge.absorb(&input);
    native_sponge
        .squeeze_native_field_elements(1)
        .first()
        .ok_or_else(|| anyhow!("Error getting the first element of the input"))
        .copied()
}

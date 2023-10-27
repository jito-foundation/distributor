from solders.pubkey import Pubkey


# get the distributor associated token address
def get_distributor_pda(mint, program_id, version=0):
    (distributor, bump) = Pubkey.find_program_address(
        [b"MerkleDistributor", bytes(mint), version.to_bytes(1, "little")], program_id
    )
    return distributor, bump

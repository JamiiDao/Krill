mod ed25519;
pub use ed25519::*;

mod state;
pub use state::*;

fn main() {
    let party1: &str = "party1@server.party";
    let party2: &str = "party2@server.party";

    println!(
        "Party1 Identifier: {:?}",
        FrostEd25519Dkg::get_identifier(party1)
    );
    println!(
        "Party2 Identifier: {:?}",
        FrostEd25519Dkg::get_identifier(party2)
    );

    let mut party1_dkg = FrostEd25519Dkg::new(party1)
        .set_maximum_signers(2)
        .set_minimum_signers(2);
    let mut party2_dkg = FrostEd25519Dkg::new(party2)
        .set_maximum_signers(2)
        .set_minimum_signers(2);
    let parties = &[party1, party2];
    party1_dkg.add_parties(parties);
    party2_dkg.add_parties(parties);

    // Round1 /////////////////
    party1_dkg.part1();
    party2_dkg.part1();

    // --- Party 1 Round1 Channel Send
    party1_dkg.add_received_round1_package(
        party2_dkg.party_id(),
        party2_dkg.round1_package().unwrap().clone(),
    );
    // --- Party 2 Round1 Channel Send
    party2_dkg.add_received_round1_package(
        party1_dkg.party_id(),
        party1_dkg.round1_package().unwrap().clone(),
    );

    // // Round2 /////////////////
    party1_dkg.part2();
    party2_dkg.part2();
    // --- Party 1 Round2 Channel Send
    let party1_round2_pacakge = party2_dkg
        .transmit_round2_package(party1_dkg.party_id())
        .unwrap()
        .clone();

    // --- Party 2 Round2 Channel Send
    let party2_round2_pacakge = party1_dkg
        .transmit_round2_package(party2_dkg.party_id())
        .unwrap()
        .clone();

    party1_dkg.add_received_round2_package(party2, party1_round2_pacakge);
    party2_dkg.add_received_round2_package(party1, party2_round2_pacakge);

    // Round3 /////////////////
    let party1_keypair = party1_dkg.part3();
    let party2_keypair = party2_dkg.part3();

    assert_eq!(
        party1_keypair.verifying_key(),
        party2_keypair.verifying_key()
    );
}

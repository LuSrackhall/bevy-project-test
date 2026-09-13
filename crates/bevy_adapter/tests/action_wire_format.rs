//! Every `Action` variant must survive the network encoding.
//!
//! The network path encodes with bincode, which is positional (not
//! self-describing) and therefore far less forgiving than the RON used for
//! replays. Before this test only `MoveTo` and `NoOp` had ever crossed the wire
//! in the e2e suites, so the other six variants could have been wire-broken with
//! every test still green — while replaying the same game worked fine.
//!
//! The corpus comes from `simulation::command::all_action_variants()`, which is
//! exhaustive by construction, so a newly added `Action` variant lands here
//! automatically instead of relying on someone remembering to extend this test.

use bevy_adapter::network::{PlayerTickFrame, RelayClientMessage};
use simulation::command::{all_action_variants, GameCommand};

fn corpus_commands() -> Vec<GameCommand> {
    all_action_variants()
        .into_iter()
        .enumerate()
        .map(|(i, action)| GameCommand {
            tick: 100 + i as u32,
            player_id: (i % 8) as u8,
            action,
        })
        .collect()
}

#[test]
fn every_action_variant_survives_bincode_round_trip() {
    let commands = corpus_commands();
    let frame = PlayerTickFrame {
        magic: 0xBEEF,
        version: 1,
        game_id: 1,
        tick: 100,
        player_id: 0,
        commands: commands.clone(),
        player_sid: 1,
    };

    // Encode the exact envelope the client uplinks, not a bare `Action`.
    let msg = RelayClientMessage::PlayerTick(frame);
    let bytes = bincode::serde::encode_to_vec(&msg, bincode::config::standard())
        .expect("bincode encode of PlayerTickFrame");
    let (decoded, _): (RelayClientMessage, _) =
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard())
            .expect("bincode decode of PlayerTickFrame");

    match decoded {
        RelayClientMessage::PlayerTick(decoded_frame) => {
            assert_eq!(decoded_frame.magic, 0xBEEF);
            assert_eq!(decoded_frame.version, 1);
            assert_eq!(decoded_frame.tick, 100);
            assert_eq!(
                decoded_frame.commands, commands,
                "bincode round-trip changed one or more Action payloads — \
                 the variant is replay-clean (RON) but wire-broken (bincode)"
            );
        }
        other => panic!("expected PlayerTick, got {other:?}"),
    }
}

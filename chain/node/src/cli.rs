//! The command-line interface.

use polkadot_sdk::*;

#[derive(Debug, clap::Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Option<Subcommand>,

    #[clap(flatten)]
    pub run: sc_cli::RunCmd,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    /// Key management: generate, inspect and insert session keys, and generate
    /// a node's network key. The devnet's first-boot script uses it to make
    /// every key on the node's own disk (`chain/docker/entrypoint.sh`).
    #[command(subcommand)]
    Key(sc_cli::KeySubcommand),

    /// Print the chain specification.
    BuildSpec(sc_cli::BuildSpecCmd),

    /// Validate blocks.
    CheckBlock(sc_cli::CheckBlockCmd),

    /// Export blocks.
    ExportBlocks(sc_cli::ExportBlocksCmd),

    /// Export the state of a given block.
    ExportState(sc_cli::ExportStateCmd),

    /// Import blocks.
    ImportBlocks(sc_cli::ImportBlocksCmd),

    /// Remove the whole chain.
    PurgeChain(sc_cli::PurgeChainCmd),

    /// Revert the chain to a previous state.
    Revert(sc_cli::RevertCmd),

    /// Database meta columns information.
    ChainInfo(sc_cli::ChainInfoCmd),
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn key(args: &[&str]) -> sc_cli::KeySubcommand {
        let argv = ["demiurge-node", "key"]
            .into_iter()
            .chain(args.iter().copied());
        match Cli::try_parse_from(argv) {
            Ok(Cli {
                subcommand: Some(Subcommand::Key(cmd)),
                ..
            }) => cmd,
            Ok(other) => panic!("`key {args:?}` parsed as something else: {other:?}"),
            Err(e) => panic!("`key {args:?}` did not parse: {e}"),
        }
    }

    /// The key commands the devnet's first-boot script runs
    /// (`chain/docker/entrypoint.sh`), with the arguments it runs them with.
    /// Each must reach `sc_cli::KeySubcommand`.
    #[test]
    fn the_key_subcommand_is_mounted() {
        assert!(matches!(
            key(&["generate", "--scheme", "sr25519", "--output-type", "json"]),
            sc_cli::KeySubcommand::Generate(_)
        ));
        assert!(matches!(
            key(&[
                "inspect",
                "--public",
                "--scheme",
                "ed25519",
                "--output-type",
                "json",
                "0x0000000000000000000000000000000000000000000000000000000000000000",
            ]),
            sc_cli::KeySubcommand::Inspect(_)
        ));
        assert!(matches!(
            key(&[
                "insert",
                "--keystore-path",
                "/data/keystore",
                "--key-type",
                "aura",
                "--scheme",
                "sr25519",
                "--suri",
                "/data/a-file-holding-the-secret",
            ]),
            sc_cli::KeySubcommand::Insert(_)
        ));
        assert!(matches!(
            key(&["generate-node-key", "--file", "/data/node-key"]),
            sc_cli::KeySubcommand::GenerateNodeKey(_)
        ));
        assert!(matches!(
            key(&["inspect-node-key", "--file", "/data/node-key"]),
            sc_cli::KeySubcommand::InspectNodeKey(_)
        ));
    }
}

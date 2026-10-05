//! Paying a checked `qor://pay` request (ADR-076, ADR-077): the transfer and its remark in one `batch_all`, approved
//! in the host dialog, signed in the vault, and finalised. What is checked before this, and why, is in `crate::pay`.

use serde::Serialize;
use subxt::dynamic::{self, Value};

use crate::error::{QorError, QorResult};
use crate::pay::{PayRequest, DEVNET_GENESIS};
use crate::vault::{normalise_address, Vault};
use crate::{Confirm, Prompt};

use super::assets::remark_call;
use super::ChainClient;

/// What a paid request reports back to the person.
#[derive(Debug, Clone, Serialize)]
pub struct PayReceipt {
    pub tx_hash: String,
    pub block_hash: String,
}

impl ChainClient {
    /// Pay a checked request from `from`. The connected chain must be Demiurge Devnet by genesis hash, whatever it
    /// calls itself (ADR-077 decision 3). The transfer and the remark that names the request are one `batch_all`, so
    /// the app never sees one without the other.
    pub async fn pay_request(
        &self,
        vault: &Vault,
        confirm: &dyn Confirm,
        from: &str,
        request: &PayRequest,
    ) -> QorResult<PayReceipt> {
        let to = normalise_address(&request.to)?;
        let connection = self.connected().await?;
        let genesis = format!("{:?}", connection.api.genesis_hash());
        if !genesis.eq_ignore_ascii_case(DEVNET_GENESIS) {
            return Err(QorError::PaymentRefused(
                "the launcher is not connected to Demiurge Devnet, so nothing was paid. Switch to the devnet in Chain \
                 settings and click the link again"
                    .into(),
            ));
        }
        let balance = self.balance(from).await?;
        let after = balance.checked_sub(request.amount_sparks).ok_or_else(|| {
            QorError::InsufficientFunds {
                have: balance.to_string(),
                need: request.amount_sparks.to_string(),
            }
        })?;

        let calls = vec![
            Value::unnamed_variant(
                "Balances",
                [Value::named_variant(
                    "transfer_keep_alive",
                    [
                        (
                            "dest",
                            Value::unnamed_variant("Id", [Value::from_bytes(to)]),
                        ),
                        ("value", Value::u128(request.amount_sparks)),
                    ],
                )],
            ),
            remark_call(&request.remark()),
        ];
        let call = dynamic::transaction(
            "Utility",
            "batch_all",
            vec![Value::unnamed_composite(calls)],
        );

        let finalised = self
            .sign_and_finalise(
                &connection,
                vault,
                confirm,
                from,
                &call,
                |chain_name, endpoint| pay_prompt(request, from, after, chain_name, endpoint),
                || Ok(()),
                "payment",
            )
            .await?;
        Ok(PayReceipt {
            tx_hash: finalised.tx_hash,
            block_hash: finalised.block_hash,
        })
    }
}

/// The dialog. Every value in it comes from the checked request and the chain, never from text the website wrote
/// except the label, which is shown as the website's words.
fn pay_prompt(
    request: &PayRequest,
    from: &str,
    after: u128,
    chain_name: &str,
    endpoint: &str,
) -> Prompt {
    let cgt = |sparks: u128| {
        format!(
            "{} {}",
            crate::cgt::format_cgt_grouped(sparks),
            crate::cgt::SYMBOL
        )
    };
    Prompt {
        title: format!("{} asks you to pay", request.app_name),
        body: format!(
            "Pay {amount}\nFor: \"{label}\" ({app}'s words)\n\nFrom: {from}\nTo: {to}\n\n\
             Fee: none. This chain charges no fee yet.\nYour balance after: {after}\n\n\
             Chain: {chain_name} (checked: Demiurge Devnet)\nNode: {endpoint}\n\n\
             The request was signed by {app}. Approving signs this payment with your key and sends it. It cannot be \
             undone.",
            amount = cgt(request.amount_sparks),
            label = request.label,
            app = request.app_name,
            to = request.to,
            after = cgt(after),
        ),
        approve: "Pay".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_dialog_names_the_app_the_amount_the_recipient_and_no_fee() {
        let request = PayRequest {
            app_id: "arqade",
            app_name: "ARQADE",
            id: "0123456789abcdef".into(),
            to: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".into(),
            amount_sparks: 5 * 10u128.pow(18),
            label: "Tip for Flux Four".into(),
            expires: 0,
        };
        let prompt = pay_prompt(
            &request,
            "5FROM",
            95 * 10u128.pow(18),
            "Demiurge Devnet",
            "wss://rpc.qorsync.dev",
        );
        assert_eq!(prompt.title, "ARQADE asks you to pay");
        assert_eq!(prompt.approve, "Pay");
        for needle in [
            "Pay 5.00 CGT",
            "Tip for Flux Four",
            request.to.as_str(),
            "5FROM",
            "Fee: none",
            "95.00 CGT",
        ] {
            assert!(prompt.body.contains(needle), "{needle} in {}", prompt.body);
        }
    }
}

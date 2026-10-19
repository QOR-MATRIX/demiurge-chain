//! The commands a shell may send the core, by name, with their arguments as JSON (ADR-104 decision 3).
//!
//! The shape is the one the Windows launcher's interface already uses with Tauri's `invoke`: a command's name, an object
//! of named arguments, and an answer that is a value or a [`QorError`]. A command moved into the core is one function
//! both shells call. What a command may never do is answer with a key: nothing here reads one.

use serde::Deserialize;
use serde_json::{json, Value};

use crate::cgt;
use crate::error::{QorError, QorResult};

/// Answer `command` with `args`.
pub fn call(command: &str, args: Value) -> QorResult<Value> {
    match command {
        "core_version" => Ok(json!({ "version": crate::VERSION })),
        "token_info" => Ok(json!({
            "symbol": cgt::SYMBOL,
            "name": cgt::NAME,
            "decimals": cgt::DECIMALS,
            "sub_unit": "Spark",
            "sub_units_per_cgt": cgt::SPARKS_PER_CGT,
            "atomic_unit": "ember",
        })),
        "parse_amount" => {
            let Text { text } = arguments(args)?;
            let embers = cgt::parse_amount(&text)?;
            Ok(
                json!({ "embers": embers.to_string(), "sparks": cgt::format_sparks(embers), "cgt": cgt::format_cgt_grouped(embers) }),
            )
        }
        "format_sparks" => {
            let Embers { embers } = arguments(args)?;
            let embers: u128 = embers.parse().map_err(|_| {
                QorError::BadAmount("embers must be a whole number written in digits".into())
            })?;
            Ok(
                json!({ "sparks": cgt::format_sparks(embers), "cgt": cgt::format_cgt_grouped(embers) }),
            )
        }
        other => Err(QorError::Internal(format!(
            "the core has no command {other:?}"
        ))),
    }
}

#[derive(Deserialize)]
struct Text {
    text: String,
}

/// Amounts cross the bridge as decimal strings: a `u128` of embers does not fit in a JavaScript or QML number.
#[derive(Deserialize)]
struct Embers {
    embers: String,
}

fn arguments<T: for<'de> Deserialize<'de>>(args: Value) -> QorResult<T> {
    serde_json::from_value(args)
        .map_err(|e| QorError::Internal(format!("the command's arguments are not right: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_core_says_its_version_and_the_units() {
        assert_eq!(
            call("core_version", Value::Null).unwrap()["version"],
            crate::VERSION
        );
        let units = call("token_info", Value::Null).unwrap();
        assert_eq!(units["symbol"], "CGT");
        assert_eq!(units["sub_units_per_cgt"], 1000);
        assert_eq!(units["atomic_unit"], "ember");
    }

    #[test]
    fn an_amount_typed_in_sparks_or_cgt_is_read_exactly() {
        let read = call("parse_amount", json!({ "text": "250 Sparks" })).unwrap();
        assert_eq!(read["embers"], (250 * cgt::SPARK).to_string());
        assert_eq!(read["sparks"], "250 Sparks");
        assert_eq!(
            call("parse_amount", json!({ "text": "1.25 CGT" })).unwrap()["sparks"],
            "1,250 Sparks"
        );
        let refused = call("parse_amount", json!({ "text": "0.0000000000000000001" })).unwrap_err();
        assert_eq!(refused.kind(), "bad_amount");
    }

    #[test]
    fn the_largest_amount_crosses_as_a_string_and_is_never_rounded() {
        let all = u128::MAX.to_string();
        let shown = call("format_sparks", json!({ "embers": all })).unwrap();
        // Finer than a Spark, so shown in full as CGT: every digit, none dropped.
        assert_eq!(
            shown["sparks"],
            format!("{} CGT", cgt::format_cgt_grouped(u128::MAX))
        );
        assert!(shown["cgt"]
            .as_str()
            .unwrap()
            .starts_with("340,282,366,920,938,463,463."));
        assert_eq!(
            call("format_sparks", json!({ "embers": "-1" }))
                .unwrap_err()
                .kind(),
            "bad_amount"
        );
    }

    #[test]
    fn an_unknown_command_or_a_wrong_argument_is_refused() {
        assert_eq!(
            call("vault_secret", Value::Null).unwrap_err().kind(),
            "internal"
        );
        assert_eq!(
            call("parse_amount", json!({ "amount": "1" }))
                .unwrap_err()
                .kind(),
            "internal"
        );
    }
}

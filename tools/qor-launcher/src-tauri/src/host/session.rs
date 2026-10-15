//! One product's session on its channel, once the operating system has said who is on the other end: the handshake,
//! then requests one at a time until the product closes the channel.

use std::time::{Duration, Instant};

use serde_json::Value;
use subtle::ConstantTimeEq;
use tokio::io::{AsyncRead, AsyncWrite};

use super::frame::{read_frame, write_frame};
use super::protocol::{code, FromProduct, ToProduct, MIN_PROTOCOL, PROTOCOL};
use super::{random_hex, Handler, HostError, ProductRecord, SpawnToken};

/// How long a spawn token is good for, from the spawn (ADR-046 decision 5: seconds).
pub const TOKEN_LIFETIME: Duration = Duration::from_secs(30);

/// What the launcher expects of the one connection a spawn allows: which product, and the token it was given. Moved
/// into [`serve`], so it is used once.
pub struct Ticket {
    pub product: ProductRecord,
    pub token: SpawnToken,
    pub issued: Instant,
}

async fn send<S: AsyncWrite + Unpin>(stream: &mut S, message: &ToProduct) -> Result<(), HostError> {
    let frame = serde_json::to_vec(message).map_err(|e| HostError::Internal(e.to_string()))?;
    write_frame(stream, &frame).await.map_err(HostError::Io)
}

async fn refuse<S: AsyncWrite + Unpin>(
    stream: &mut S,
    code: &str,
    message: &str,
    error: HostError,
) -> HostError {
    // The product is told why; if it is not listening any more, the refusal stands all the same.
    let _ = send(
        stream,
        &ToProduct::Error {
            id: None,
            code: code.into(),
            message: message.into(),
        },
    )
    .await;
    error
}

/// The protocol both sides speak: the newest this launcher has that the product accepts, if any.
fn agree(protocol: u32, min_protocol: u32) -> Option<u32> {
    let chosen = protocol.min(PROTOCOL);
    (chosen >= min_protocol.max(MIN_PROTOCOL)).then_some(chosen)
}

/// Run one session to its end. Returns `Ok` when the product closes the channel after a good handshake.
pub async fn serve<S, H>(mut stream: S, ticket: Ticket, handler: &H) -> Result<(), HostError>
where
    S: AsyncRead + AsyncWrite + Unpin,
    H: Handler,
{
    let Ticket {
        product,
        token,
        issued,
    } = ticket;

    // The hello, within what is left of the token's lifetime.
    let left = TOKEN_LIFETIME.saturating_sub(issued.elapsed());
    let frame = match tokio::time::timeout(left, read_frame(&mut stream)).await {
        Err(_) => return Err(HostError::Expired),
        Ok(Err(e)) => return Err(HostError::Io(e)),
        Ok(Ok(None)) => return Err(HostError::Closed),
        Ok(Ok(Some(frame))) => frame,
    };
    let hello = serde_json::from_slice::<FromProduct>(&frame);
    let Ok(FromProduct::Hello {
        protocol,
        min_protocol,
        product: id,
        version,
        token: offered,
    }) = hello
    else {
        return Err(refuse(
            &mut stream,
            code::BAD_REQUEST,
            "the first frame must be a hello",
            HostError::BadHandshake("the first frame was not a hello".into()),
        )
        .await);
    };
    // Constant time, so how much of a guess was right is not measurable; and once, since the ticket is gone after
    // this call whatever its outcome.
    let token_matches: bool = offered.as_bytes().ct_eq(token.hex().as_bytes()).into();
    if !token_matches || issued.elapsed() > TOKEN_LIFETIME {
        return Err(refuse(
            &mut stream,
            code::NOT_AUTHORISED,
            "this connection was not the one the launcher started",
            HostError::BadToken,
        )
        .await);
    }
    if id != product.id {
        return Err(refuse(
            &mut stream,
            code::NOT_AUTHORISED,
            "this connection was not the one the launcher started",
            HostError::BadHandshake(format!(
                "a hello from {id:?}, where {} was started",
                product.id
            )),
        )
        .await);
    }
    let Some(chosen) = agree(protocol, min_protocol) else {
        return Err(refuse(
            &mut stream,
            code::PROTOCOL_MISMATCH,
            &format!("this launcher speaks the QOR host protocol {MIN_PROTOCOL} to {PROTOCOL}"),
            HostError::BadHandshake(format!(
                "{} asked for protocol {min_protocol} to {protocol}",
                product.name
            )),
        )
        .await);
    };

    send(
        &mut stream,
        &ToProduct::Welcome {
            protocol: chosen,
            capabilities: handler.capabilities().to_vec(),
            session: random_hex(16),
        },
    )
    .await?;
    tracing::info!(product = product.id, %version, protocol = chosen, "a product the launcher started connected");

    loop {
        let Some(frame) = read_frame(&mut stream).await.map_err(HostError::Io)? else {
            tracing::info!(product = product.id, "a product closed its channel");
            return Ok(());
        };
        let (id, method, params) = match serde_json::from_slice::<FromProduct>(&frame) {
            Ok(FromProduct::Request { id, method, params }) => (id, method, params),
            Ok(FromProduct::Hello { .. }) => {
                return Err(refuse(
                    &mut stream,
                    code::BAD_REQUEST,
                    "a session has one hello",
                    HostError::BadHandshake("a second hello".into()),
                )
                .await)
            }
            Err(e) => {
                return Err(refuse(
                    &mut stream,
                    code::BAD_REQUEST,
                    "a frame that is not a request",
                    HostError::BadHandshake(format!("an unreadable frame: {e}")),
                )
                .await)
            }
        };

        let answer = if handler.capabilities().contains(&method.as_str()) {
            handler.handle(&method, params).await
        } else {
            Err(super::Refusal::new(
                code::UNSUPPORTED_METHOD,
                format!("this launcher does not offer {method:?}"),
            ))
        };
        let reply = match answer {
            Ok(result) => ToProduct::Result { id, result },
            Err(refusal) => {
                // The message too: the code alone left a refused publish unexplained (15 October 2026). It never holds the
                // token or the session id, which `nothing_secret_reaches_the_log_at_any_level` checks.
                tracing::info!(
                    product = product.id,
                    %method,
                    code = %refusal.code,
                    message = %refusal.message,
                    "a product's request was refused"
                );
                ToProduct::Error {
                    id: Some(id),
                    code: refusal.code,
                    message: refusal.message,
                }
            }
        };
        send(&mut stream, &reply).await?;
    }
}

/// A request's parameters as one type, or the refusal the product reads.
pub fn params<T: serde::de::DeserializeOwned>(params: Value) -> Result<T, super::Refusal> {
    serde_json::from_value(params)
        .map_err(|e| super::Refusal::new(code::BAD_REQUEST, format!("{e}")))
}

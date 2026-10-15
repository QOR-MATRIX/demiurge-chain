//! Frames on the channel: a 4-byte big-endian length, then that many bytes of UTF-8 JSON, at most [`FRAME_LIMIT`]
//! (ADR-046 decision 3). A longer frame closes the connection, and nothing is allocated from a length the peer gave
//! until it has been checked.

use std::io;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// The largest frame either side sends or accepts: 1 MiB.
pub const FRAME_LIMIT: usize = 1 << 20;

/// The next frame, or `None` when the peer closed the channel between frames.
pub async fn read_frame<R: AsyncRead + Unpin>(reader: &mut R) -> io::Result<Option<Vec<u8>>> {
    let mut length = [0u8; 4];
    match reader.read_exact(&mut length).await {
        Ok(_) => {}
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    }
    let length = u32::from_be_bytes(length) as usize;
    if length > FRAME_LIMIT {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("a frame of {length} bytes, over the {FRAME_LIMIT}-byte limit"),
        ));
    }
    let mut frame = vec![0u8; length];
    reader.read_exact(&mut frame).await?;
    Ok(Some(frame))
}

/// Send one frame. A frame over the limit is refused here rather than sent.
pub async fn write_frame<W: AsyncWrite + Unpin>(writer: &mut W, frame: &[u8]) -> io::Result<()> {
    if frame.len() > FRAME_LIMIT {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "a frame of {} bytes, over the {FRAME_LIMIT}-byte limit",
                frame.len()
            ),
        ));
    }
    writer
        .write_all(&(frame.len() as u32).to_be_bytes())
        .await?;
    writer.write_all(frame).await?;
    writer.flush().await
}

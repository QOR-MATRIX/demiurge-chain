//! The bridge: the Qt shell's one way into the core (ADR-104 decision 3). Its C declarations are
//! `include/qor_core.h`.
//!
//! The shell sends one JSON object, `{"command": "...", "args": {...}}`, and gets one back: `{"ok": <answer>}`, or
//! `{"error": {"kind": "...", "message": "..."}}` as [`QorError`] serialises. Every string crosses as UTF-8 with a
//! terminating zero. What the core hands out, the shell gives back to [`qor_core_free`]; nothing else is freed there.
//!
//! This is the only module in the core allowed `unsafe` code, and it uses it for one thing: reading a C string it was
//! given and handing back one it made. A null or non-UTF-8 request is answered with an error, never read past.

#![allow(unsafe_code)]

use std::ffi::{c_char, CStr, CString};

use serde_json::{json, Value};

use crate::error::QorError;

/// The core's version, a static string the shell must not free.
#[no_mangle]
pub extern "C" fn qor_core_version() -> *const c_char {
    static VERSION: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();
    VERSION.as_ptr().cast()
}

/// Answer one request. The answer is never null; the shell frees it with [`qor_core_free`].
///
/// # Safety
/// `request` is null or points at a zero-terminated string that stays valid for the call.
#[no_mangle]
pub unsafe extern "C" fn qor_core_call(request: *const c_char) -> *mut c_char {
    let answer = if request.is_null() {
        refused(QorError::Internal("the bridge was given no request".into()))
    } else {
        // SAFETY: the caller promises a zero-terminated string valid for this call; it is only read.
        let text = unsafe { CStr::from_ptr(request) };
        answer(text.to_str())
    };
    // A JSON text has no zero byte in it (serde_json escapes U+0000), so this cannot fail; if it ever did, the shell is
    // told rather than handed nothing.
    CString::new(answer.to_string())
        .unwrap_or_else(|_| {
            CString::new(r#"{"error":{"kind":"internal","message":"unanswerable"}}"#)
                .expect("no zero byte")
        })
        .into_raw()
}

/// Give back an answer [`qor_core_call`] made. Null is ignored.
///
/// # Safety
/// `answer` is null or a pointer [`qor_core_call`] returned, not yet freed.
#[no_mangle]
pub unsafe extern "C" fn qor_core_free(answer: *mut c_char) {
    if !answer.is_null() {
        // SAFETY: the caller promises this came from `CString::into_raw` in `qor_core_call` and is freed once.
        drop(unsafe { CString::from_raw(answer) });
    }
}

fn answer(request: Result<&str, std::str::Utf8Error>) -> Value {
    let Ok(text) = request else {
        return refused(QorError::Internal("the request is not UTF-8".into()));
    };
    let Ok(request) = serde_json::from_str::<Value>(text) else {
        return refused(QorError::Internal("the request is not JSON".into()));
    };
    let Some(command) = request.get("command").and_then(Value::as_str) else {
        return refused(QorError::Internal("the request names no command".into()));
    };
    let args = request.get("args").cloned().unwrap_or(Value::Null);
    match crate::commands::call(command, args) {
        Ok(value) => json!({ "ok": value }),
        Err(error) => refused(error),
    }
}

fn refused(error: QorError) -> Value {
    json!({ "error": error })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What the shell does: send a request, read the answer, give it back.
    fn round_trip(request: &[u8]) -> Value {
        let request = CString::new(request).unwrap();
        let answer = unsafe { qor_core_call(request.as_ptr()) };
        assert!(!answer.is_null());
        let text = unsafe { CStr::from_ptr(answer) }
            .to_str()
            .unwrap()
            .to_owned();
        unsafe { qor_core_free(answer) };
        serde_json::from_str(&text).unwrap()
    }

    #[test]
    fn a_command_crosses_and_its_answer_comes_back() {
        let answer = round_trip(br#"{"command":"parse_amount","args":{"text":"250 Sparks"}}"#);
        assert_eq!(answer["ok"]["sparks"], "250 Sparks");
        let version = unsafe { CStr::from_ptr(qor_core_version()) }
            .to_str()
            .unwrap();
        assert_eq!(version, crate::VERSION);
    }

    #[test]
    fn a_refusal_comes_back_as_the_error_the_interface_already_reads() {
        let answer = round_trip(br#"{"command":"parse_amount","args":{"text":"lots"}}"#);
        assert_eq!(answer["error"]["kind"], "bad_amount");
        assert!(answer["error"]["message"].is_string());
        assert!(answer.get("ok").is_none());
    }

    #[test]
    fn nothing_malformed_is_read_past() {
        assert_eq!(round_trip(b"not json")["error"]["kind"], "internal");
        assert_eq!(round_trip(br#"{"args":{}}"#)["error"]["kind"], "internal");
        assert!(round_trip(b"\xff\xfe")["error"]["message"]
            .as_str()
            .unwrap()
            .contains("not UTF-8"));
        let answer = unsafe { qor_core_call(std::ptr::null()) };
        let text = unsafe { CStr::from_ptr(answer) }
            .to_str()
            .unwrap()
            .to_owned();
        unsafe { qor_core_free(answer) };
        assert!(text.contains("no request"));
        unsafe { qor_core_free(std::ptr::null_mut()) };
    }
}

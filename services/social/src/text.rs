//! What people write, made safe to show before it is kept (docs/blueprints/social.md section 4).
//!
//! NFKC-normalised, so look-alike forms collapse to one; control characters removed, except a line break in a
//! message; bidirectional overrides and isolates removed, so a name or a message cannot reorder what is shown after
//! it; and trimmed. Limits count characters, not bytes.

use unicode_normalization::UnicodeNormalization as _;

use crate::error::{SocialError, SocialResult};

/// The longest message, in characters.
pub const MESSAGE_MAX: usize = 4_000;

fn is_bidi_control(c: char) -> bool {
    matches!(c, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{200E}' | '\u{200F}' | '\u{061C}')
}

fn clean(input: &str, keep_newlines: bool) -> String {
    let normalised: String = input.replace("\r\n", "\n").nfkc().collect();
    let kept: String = normalised
        .chars()
        .map(|c| match c {
            '\r' => '\n',
            '\n' | '\t' if !keep_newlines => ' ',
            c => c,
        })
        .filter(|&c| {
            if is_bidi_control(c) {
                return false;
            }
            if c == '\n' {
                return keep_newlines;
            }
            !c.is_control() || (c == '\t' && keep_newlines)
        })
        .collect();
    if keep_newlines {
        // At most two line breaks in a row: a message is not a way to push others' messages off the screen.
        let mut out = String::with_capacity(kept.len());
        let mut breaks = 0;
        for c in kept.chars() {
            if c == '\n' {
                breaks += 1;
                if breaks > 2 {
                    continue;
                }
            } else {
                breaks = 0;
            }
            out.push(c);
        }
        out.trim().to_string()
    } else {
        kept.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

/// A message body: 1 to [`MESSAGE_MAX`] characters once cleaned.
pub fn message(input: &str) -> SocialResult<String> {
    let body = clean(input, true);
    let length = body.chars().count();
    if length == 0 {
        return Err(SocialError::bad("a message needs some words"));
    }
    if length > MESSAGE_MAX {
        return Err(SocialError::bad(format!(
            "a message is at most {MESSAGE_MAX} characters; this one is {length}"
        )));
    }
    Ok(body)
}

/// A one-line field (a room's name, a topic, a display line, a status): at most `max` characters once cleaned, and
/// at least `min`.
pub fn line(input: &str, what: &str, min: usize, max: usize) -> SocialResult<String> {
    let text = clean(input, false);
    let length = text.chars().count();
    if length < min || length > max {
        return Err(SocialError::bad(if min > 0 {
            format!("{what} is {min} to {max} characters")
        } else {
            format!("{what} is at most {max} characters")
        }));
    }
    Ok(text)
}

/// A bio: several lines, at most `max` characters.
pub fn paragraph(input: &str, what: &str, max: usize) -> SocialResult<String> {
    let text = clean(input, true);
    if text.chars().count() > max {
        return Err(SocialError::bad(format!(
            "{what} is at most {max} characters"
        )));
    }
    Ok(text)
}

/// A reaction: one emoji or a short sequence of them (skin tones, joiners), never words.
pub fn emoji(input: &str) -> SocialResult<String> {
    let e: String = input.nfc().collect();
    let pictographic = |c: char| {
        matches!(c as u32,
            0x1F000..=0x1FAFF | 0x2600..=0x27BF | 0x2B00..=0x2BFF | 0x2190..=0x21FF | 0x2300..=0x23FF
            | 0x3030 | 0x303D | 0x3297 | 0x3299 | 0x00A9 | 0x00AE | 0x203C | 0x2049 | 0x2122 | 0x2139)
    };
    let joiner = |c: char| matches!(c as u32, 0x200D | 0xFE0F | 0xFE0E | 0x20E3 | 0x1F3FB..=0x1F3FF | 0xE0020..=0xE007F);
    let count = e.chars().count();
    // A keycap (`#️⃣`, `1️⃣`) is a character and U+20E3, with nothing pictographic in it.
    let keycap = e.ends_with('\u{20E3}');
    if count == 0 || count > 10 || e.len() > 32 || !(keycap || e.chars().any(pictographic)) {
        return Err(SocialError::bad("a reaction is an emoji"));
    }
    if !e
        .chars()
        .all(|c| pictographic(c) || joiner(c) || c.is_ascii_digit() || c == '#' || c == '*')
    {
        return Err(SocialError::bad("a reaction is an emoji"));
    }
    Ok(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_message_is_cleaned_before_it_is_kept() {
        assert_eq!(message("  hello  ").unwrap(), "hello");
        assert_eq!(message("ｆｕｌｌ width").unwrap(), "full width", "NFKC");
        assert_eq!(
            message("a\u{202E}evil").unwrap(),
            "aevil",
            "no right-to-left override"
        );
        assert_eq!(message("a\u{2066}b\u{2069}").unwrap(), "ab", "no isolates");
        assert_eq!(message("bell\u{7}").unwrap(), "bell");
        assert_eq!(message("one\r\ntwo\rthree").unwrap(), "one\ntwo\nthree");
        assert_eq!(
            message("a\n\n\n\n\nb").unwrap(),
            "a\n\nb",
            "at most two breaks"
        );
        assert!(message(" \u{202E} \n ").is_err(), "nothing left");
        assert!(message(&"x".repeat(MESSAGE_MAX)).is_ok());
        assert!(message(&"x".repeat(MESSAGE_MAX + 1)).is_err());
        assert!(
            message(&"é".repeat(MESSAGE_MAX)).is_ok(),
            "characters, not bytes"
        );
    }

    #[test]
    fn a_line_is_one_line() {
        assert_eq!(
            line(" The\nLounge\t ", "a name", 1, 48).unwrap(),
            "The Lounge"
        );
        assert!(line("", "a name", 1, 48).is_err());
        assert!(line(&"n".repeat(49), "a name", 1, 48).is_err());
        assert_eq!(line("", "a status", 0, 80).unwrap(), "");
    }

    #[test]
    fn a_reaction_is_an_emoji_and_nothing_else() {
        for ok in ["👍", "❤️", "👍🏽", "👨‍👩‍👧", "🔥", "#️⃣"] {
            assert_eq!(emoji(ok).unwrap(), ok, "{ok}");
        }
        for bad in ["", "lol", "a👍", "👍 ", "<script>", &"👍".repeat(11)] {
            assert!(emoji(bad).is_err(), "{bad:?}");
        }
    }
}

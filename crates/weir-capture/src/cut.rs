//! The cut: an own excerpt shortened at a line, with one closing line
//! saying what was left out, to keep within the limit (20; core.12). It is
//! the only cut: every host cuts with it, so every host cuts the same way
//! (24).

const OPEN: &str = "[cut: ";
const CLOSE: &str = " left out, banked whole with the capture]";

/// The smallest limit a host may set: the closing line's length with both
/// numbers at their widest, so the closing line always fits.
pub const SMALLEST_LIMIT: usize =
    OPEN.len() + WIDEST + " lines and ".len() + WIDEST + " bytes".len() + CLOSE.len();

/// The digits of the largest count, `u64::MAX`.
const WIDEST: usize = 20;

/// What a cut left out of a text: its lines, a last line with no `\n`
/// counted, and its bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LeftOut {
    pub lines: u64,
    pub bytes: u64,
}

/// An excerpt as cut: the text, and what was left out when it was cut.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cut {
    pub text: String,
    pub left_out: Option<LeftOut>,
}

/// The line the host adds to a cut excerpt (20).
pub fn closing_line(left_out: LeftOut) -> String {
    let LeftOut { lines, bytes } = left_out;
    let lines_word = if lines == 1 { "line" } else { "lines" };
    let bytes_word = if bytes == 1 { "byte" } else { "bytes" };
    format!("{OPEN}{lines} {lines_word} and {bytes} {bytes_word}{CLOSE}")
}

/// `text` kept within `limit` bytes. A text within the limit is kept whole,
/// with nothing added. A longer one keeps the longest start ending just
/// after a `\n` that fits together with the closing line, then the closing
/// line; when not even the first line fits, the closing line alone. Lines
/// end at `\n`, so a cut never splits a character. Every limit of at least
/// [`SMALLEST_LIMIT`] gives an excerpt within it.
pub fn cut(text: &str, limit: usize) -> Cut {
    if text.len() <= limit {
        return Cut {
            text: text.to_string(),
            left_out: None,
        };
    }
    let all_lines = count_lines(text);
    let left_out_after = |end: usize, lines_kept: u64| LeftOut {
        lines: all_lines - lines_kept,
        bytes: (text.len() - end) as u64,
    };
    let mut kept = (0, left_out_after(0, 0));
    for (lines_kept, (newline, _)) in (1..).zip(text.match_indices('\n')) {
        let end = newline + 1;
        if end >= limit {
            break;
        }
        let left_out = left_out_after(end, lines_kept);
        if end + closing_line(left_out).len() <= limit {
            kept = (end, left_out);
        }
    }
    let (end, left_out) = kept;
    Cut {
        text: format!("{}{}", &text[..end], closing_line(left_out)),
        left_out: Some(left_out),
    }
}

/// A text's lines: each `\n` ends one, and a last line with no `\n` counts.
fn count_lines(text: &str) -> u64 {
    let ended = text.matches('\n').count() as u64;
    if text.is_empty() || text.ends_with('\n') {
        ended
    } else {
        ended + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIMIT: usize = 8192;

    /// The closing line a cut ends with, parsed back to its numbers.
    fn closing_numbers(excerpt: &str) -> (u64, u64) {
        let closing = excerpt.rsplit('\n').next().unwrap();
        let words: Vec<&str> = closing.split(' ').collect();
        assert_eq!(words[0], "[cut:", "{closing}");
        (words[1].parse().unwrap(), words[4].parse().unwrap())
    }

    fn short_lines(bytes: usize) -> String {
        let mut text = String::new();
        let mut n = 0;
        while text.len() < bytes {
            text.push_str(&format!("line {n} of the release notes\n"));
            n += 1;
        }
        text
    }

    #[test]
    fn the_smallest_limit_fits_the_widest_closing_line() {
        let widest = closing_line(LeftOut {
            lines: u64::MAX,
            bytes: u64::MAX,
        });
        assert_eq!(widest.len(), SMALLEST_LIMIT);
        assert_eq!(u64::MAX.to_string().len(), WIDEST);
    }

    #[test]
    fn the_closing_line_says_one_line_and_one_byte() {
        assert_eq!(
            closing_line(LeftOut { lines: 1, bytes: 1 }),
            "[cut: 1 line and 1 byte left out, banked whole with the capture]"
        );
        assert_eq!(
            closing_line(LeftOut {
                lines: 3,
                bytes: 120
            }),
            "[cut: 3 lines and 120 bytes left out, banked whole with the capture]"
        );
    }

    #[test]
    fn a_text_within_the_limit() {
        let text = "x".repeat(299) + "\n";
        assert_eq!(text.len(), 300);
        assert_eq!(
            cut(&text, LIMIT),
            Cut {
                text: text.clone(),
                left_out: None
            }
        );
        let exact = "y".repeat(LIMIT);
        assert_eq!(cut(&exact, LIMIT).text, exact);
    }

    #[test]
    fn a_forty_kibibyte_text() {
        let text = short_lines(40 * 1024);
        let excerpt = cut(&text, LIMIT);
        assert!(excerpt.text.len() <= LIMIT, "{}", excerpt.text.len());
        let left_out = excerpt.left_out.expect("a cut");
        let closing = closing_line(left_out);
        let kept = excerpt
            .text
            .strip_suffix(&closing)
            .expect("the closing line");
        assert!(kept.ends_with('\n'));
        assert!(text.starts_with(kept));
        let rest = &text[kept.len()..];
        assert_eq!(left_out.bytes, rest.len() as u64);
        assert_eq!(left_out.lines, count_lines(rest));
        // The longest that fits: one more line would not.
        let next_line = rest.split_inclusive('\n').next().unwrap();
        let longer = LeftOut {
            lines: left_out.lines - 1,
            bytes: left_out.bytes - next_line.len() as u64,
        };
        assert!(kept.len() + next_line.len() + closing_line(longer).len() > LIMIT);
    }

    #[test]
    fn a_first_line_longer_than_the_limit() {
        let text = "z".repeat(10000) + "\nthe second line\n";
        let excerpt = cut(&text, LIMIT);
        let left_out = LeftOut {
            lines: 2,
            bytes: text.len() as u64,
        };
        assert_eq!(excerpt.text, closing_line(left_out));
        assert_eq!(excerpt.left_out, Some(left_out));
    }

    #[test]
    fn the_cut_records_what_the_closing_line_says() {
        let text = short_lines(20 * 1024);
        let excerpt = cut(&text, LIMIT);
        let left_out = excerpt.left_out.unwrap();
        assert_eq!(
            closing_numbers(&excerpt.text),
            (left_out.lines, left_out.bytes)
        );
    }

    #[test]
    fn a_text_with_no_final_newline() {
        let text = format!("{}the last line, unended", short_lines(9000));
        let excerpt = cut(&text, LIMIT);
        let left_out = excerpt.left_out.unwrap();
        let kept = excerpt.text.strip_suffix(&closing_line(left_out)).unwrap();
        let rest = &text[kept.len()..];
        assert!(rest.ends_with("unended"));
        assert_eq!(left_out.lines, rest.matches('\n').count() as u64 + 1);
        assert_eq!(left_out.bytes, rest.len() as u64);

        // One line left out, with no newline of its own.
        let text = format!("{}\n{}", "a".repeat(10), "b".repeat(200));
        let excerpt = cut(&text, SMALLEST_LIMIT);
        assert_eq!(
            excerpt.left_out,
            Some(LeftOut {
                lines: 1,
                bytes: 200
            })
        );
        assert!(excerpt
            .text
            .starts_with("aaaaaaaaaa\n[cut: 1 line and 200 bytes"));
    }

    #[test]
    fn multi_byte_characters_near_the_limit() {
        // Lines of two-byte characters, so a limit falls inside one.
        let line = "é".repeat(100) + "\n";
        let text = line.repeat(100);
        for limit in [LIMIT - 1, LIMIT, 1001, 1002, SMALLEST_LIMIT + 1] {
            let excerpt = cut(&text, limit);
            assert!(excerpt.text.len() <= limit, "{limit}");
            let left_out = excerpt.left_out.unwrap();
            let kept = excerpt.text.strip_suffix(&closing_line(left_out)).unwrap();
            assert!(kept.is_empty() || kept.ends_with('\n'), "{limit}");
            assert_eq!(kept.len() % line.len(), 0, "{limit}");
        }
        let fits = "é".repeat(LIMIT / 2);
        assert_eq!(cut(&fits, LIMIT).text, fits);
    }
}

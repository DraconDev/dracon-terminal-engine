//! Text handling utilities with proper Unicode grapheme cluster awareness.
//!
//! This module provides functions for working with Unicode grapheme clusters,
//! which are the units of text that users perceive as single characters.

/// Returns the visual width of a character in terminal cells.
///
/// This handles:
/// - Base characters (width 1)
/// - Combining marks (width 0)
/// - Zero-width joiners (width 0)
/// - Regional indicator symbols (width 2 when part of flag pairs)
/// - Wide characters like CJK (width 2)
/// - Tone modifiers and skin tone modifiers (width 0)
///
/// For most characters, returns 1. Returns 0 for invisible/combining characters.
pub fn grapheme_width(c: char) -> u8 {
    // Regional indicator symbols: U+1F1E6 to U+1F1FF
    // These have width 0 individually but form width-2 pairs (flags)
    if matches!(c, '\u{1F1E6}'..='\u{1F1FF}') {
        return 2;
    }

    // Zero-width characters
    if c == '\u{200D}' // Zero Width Joiner (ZWJ)
        || c == '\u{200C}' // Zero Width Non-Joiner (ZWNJ)
        || c == '\u{200B}' // Zero Width Space
        || c == '\u{FEFF}' // Byte Order Mark
        || c == '\u{2060}'
    // Word Joiner
    {
        return 0;
    }

    // Tone modifiers (skin tones) and other emoji modifiers: U+1F3FB to U+1F3FF
    if matches!(
        c,
        '\u{1F3FB}'..='\u{1F3FF}'
            | '\u{1F9B0}'..='\u{1F9B3}'
            | '\u{1F9C0}'
            | '\u{1F9D0}'..='\u{1F9D6}'
            | '\u{1F9D7}'..='\u{1F9DF}'
    ) {
        return 0;
    }

    // Combining marks: Various Unicode ranges
    if matches!(
        c,
        '\u{0300}'..='\u{036F}' // General combining marks
            | '\u{1DC0}'..='\u{1DFF}' // Combining diacritical marks extended
            | '\u{20D0}'..='\u{20FF}' // Combining diacritical marks for symbols
            | '\u{FE20}'..='\u{FE2F}' // Combining half marks
    ) {
        return 0;
    }

    // Hiragana/katakana combining marks: U+3099 to U+309A
    if matches!(c, '\u{3099}'..='\u{309A}') {
        return 0;
    }

    // Wide characters (CJK): width 2
    if is_wide_char(c) {
        return 2;
    }

    // Most emoji and other characters are narrow (width 1)
    // Use unicode-width as fallback
    use unicode_width::UnicodeWidthChar;
    UnicodeWidthChar::width(c).unwrap_or(1) as u8
}

/// Check if a character is a wide character (CJK, etc.)
fn is_wide_char(c: char) -> bool {
    matches!(
        c,
        '\u{2E80}'..='\u{303E}'
            | '\u{3040}'..='\u{309F}'
            | '\u{30A0}'..='\u{30FF}'
            | '\u{3100}'..='\u{312F}'
            | '\u{3130}'..='\u{318F}'
            | '\u{3190}'..='\u{319F}'
            | '\u{31A0}'..='\u{31BF}'
            | '\u{31C0}'..='\u{31EF}'
            | '\u{31F0}'..='\u{31FF}'
            | '\u{3200}'..='\u{32FF}'
            | '\u{3300}'..='\u{4DBF}'
            | '\u{4E00}'..='\u{9FFF}'
            | '\u{A000}'..='\u{A48C}'
            | '\u{A490}'..='\u{A4CF}'
            | '\u{F900}'..='\u{FAFF}'
            | '\u{20000}'..='\u{2A6DF}'
            | '\u{2A700}'..='\u{2B73F}'
            | '\u{2B740}'..='\u{2B81F}'
            | '\u{2B820}'..='\u{2CEAF}'
            | '\u{2CEB0}'..='\u{2EBEF}'
            | '\u{30000}'..='\u{3134F}'
    )
}

/// Returns an iterator over grapheme clusters in the text.
///
/// Each item is a tuple of (byte_offset, visual_column) where:
/// - byte_offset is the starting byte position of the grapheme cluster
/// - visual_column is the cumulative visual width position
///
/// This properly handles:
/// - Base characters and emoji
/// - Combining marks (included in cluster but contribute 0 width)
/// - Zero-width joiners (ZWJ) for emoji sequences
/// - Regional indicator pairs (forming flag emojis)
/// - Skin tone modifiers
/// - Wide characters (CJK)
pub fn grapheme_indices(text: &str) -> Vec<(usize, usize)> {
    let mut result = Vec::with_capacity(text.len() / 2);
    let mut byte_offset = 0usize;
    let mut visual_column = 0usize;

    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        let char_len = c.len_utf8();
        let width = grapheme_width(c);

        // Handle regional indicator symbols (U+1F1E6 to U+1F1FF)
        // These form 2-cell flag emojis when in pairs
        if matches!(c, '\u{1F1E6}'..='\u{1F1FF}') {
            // Check if there's a second regional indicator following
            if let Some(&next_c) = chars.peek() {
                if matches!(next_c, '\u{1F1E6}'..='\u{1F1FF}') {
                    // Flag emoji: consume both RIs together
                    chars.next();
                    result.push((byte_offset, visual_column));
                    byte_offset += char_len + next_c.len_utf8();
                    visual_column += 2;
                    continue;
                }
            }
            // Single RI without pair: still a cluster
            result.push((byte_offset, visual_column));
            byte_offset += char_len;
            continue;
        }

        // ZWJ only appears in sequences - skip it here, it will be consumed below
        if c == '\u{200D}' {
            byte_offset += char_len;
            continue;
        }

        // Zero-width characters (combining marks) are part of previous cluster
        if width == 0 {
            byte_offset += char_len;
            continue;
        }

        // This is a base character starting a new grapheme cluster
        let cluster_start_byte = byte_offset;
        let cluster_start_visual = visual_column;

        // Push the cluster start
        result.push((cluster_start_byte, cluster_start_visual));

        // Advance past this character
        byte_offset += char_len;
        visual_column += width as usize;

        // Now consume any modifiers/ZWJ sequences that belong to this cluster
        while let Some(&next_c) = chars.peek() {
            // ZWJ: consume it and the following emoji as part of this cluster
            if next_c == '\u{200D}' {
                chars.next(); // consume ZWJ
                byte_offset += 3; // ZWJ is 3 bytes

                // Next character after ZWJ should be an emoji - consume it
                if let Some(&emoji_c) = chars.peek() {
                    let emoji_len = emoji_c.len_utf8();
                    chars.next();
                    byte_offset += emoji_len;
                    visual_column += grapheme_width(emoji_c) as usize;
                }
                continue;
            }

            // Skin tone modifiers extend the cluster
            if matches!(next_c, '\u{1F3FB}'..='\u{1F3FF}') {
                let next_len = next_c.len_utf8();
                chars.next();
                byte_offset += next_len;
                // Modifiers have width 0
                continue;
            }

            // Zero-width combining marks extend the cluster
            if grapheme_width(next_c) == 0 {
                let next_len = next_c.len_utf8();
                chars.next();
                byte_offset += next_len;
                continue;
            }

            // Next character starts a new cluster
            break;
        }
    }

    result
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grapheme_width_ascii() {
        assert_eq!(grapheme_width('a'), 1);
        assert_eq!(grapheme_width('Z'), 1);
        assert_eq!(grapheme_width(' '), 1);
        assert_eq!(grapheme_width('!'), 1);
    }

    #[test]
    fn test_grapheme_width_zero_width() {
        // Zero-width characters
        assert_eq!(grapheme_width('\u{200D}'), 0); // ZWJ
        assert_eq!(grapheme_width('\u{200C}'), 0); // ZWNJ
        assert_eq!(grapheme_width('\u{200B}'), 0); // Zero-width space
        assert_eq!(grapheme_width('\u{FEFF}'), 0); // BOM
    }

    #[test]
    fn test_grapheme_width_combining_marks() {
        // Combining diacritical marks
        assert_eq!(grapheme_width('\u{0300}'), 0); // Combining grave accent
        assert_eq!(grapheme_width('\u{0301}'), 0); // Combining acute accent
        assert_eq!(grapheme_width('\u{0327}'), 0); // Combining cedilla
        assert_eq!(grapheme_width('\u{036F}'), 0); // Last general combining mark

        // Hiragana/katakana voicing marks
        assert_eq!(grapheme_width('\u{3099}'), 0); // Combining katakana-hiragana voiced mark
        assert_eq!(grapheme_width('\u{309A}'), 0); // Combining katakana-hiragana semi-voiced mark
    }

    #[test]
    fn test_grapheme_width_tone_modifiers() {
        // Skin tone modifiers
        assert_eq!(grapheme_width('\u{1F3FB}'), 0); // Light skin tone
        assert_eq!(grapheme_width('\u{1F3FC}'), 0); // Medium-light skin tone
        assert_eq!(grapheme_width('\u{1F3FD}'), 0); // Medium skin tone
        assert_eq!(grapheme_width('\u{1F3FE}'), 0); // Medium-dark skin tone
        assert_eq!(grapheme_width('\u{1F3FF}'), 0); // Dark skin tone
    }

    #[test]
    fn test_grapheme_width_regional_indicators() {
        // Regional indicator symbols should return 2
        // (they form pairs for flags)
        assert_eq!(grapheme_width('\u{1F1FA}'), 2); // Letter U (USA flag)
        assert_eq!(grapheme_width('\u{1F1EB}'), 2); // Letter F (France flag)
    }

    #[test]
    fn test_grapheme_width_cjk() {
        // CJK characters are wide
        assert_eq!(grapheme_width('日'), 2);
        assert_eq!(grapheme_width('本'), 2);
        assert_eq!(grapheme_width('語'), 2);
        assert_eq!(grapheme_width('あ'), 2);
        assert_eq!(grapheme_width('ア'), 2);
    }

    #[test]
    fn test_grapheme_indices_basic() {
        let indices = grapheme_indices("hello");
        assert_eq!(indices.len(), 5);
        // Check byte offsets
        assert_eq!(indices[0].0, 0); // h
        assert_eq!(indices[1].0, 1); // e
        assert_eq!(indices[2].0, 2); // l
        assert_eq!(indices[3].0, 3); // l
        assert_eq!(indices[4].0, 4); // o
                                     // Check visual columns
        assert_eq!(indices[0].1, 0);
        assert_eq!(indices[1].1, 1);
        assert_eq!(indices[2].1, 2);
        assert_eq!(indices[3].1, 3);
        assert_eq!(indices[4].1, 4);
    }

    #[test]
    fn test_grapheme_indices_cjk() {
        let indices = grapheme_indices("日本語");
        // 3 CJK characters, each 2 cells wide
        assert_eq!(indices.len(), 3);
        assert_eq!(indices[0].0, 0);
        assert_eq!(indices[0].1, 0); // 日: starts at col 0
        assert_eq!(indices[1].1, 2); // 本: starts at col 2
        assert_eq!(indices[2].1, 4); // 語: starts at col 4
    }

    #[test]
    fn test_grapheme_indices_combining_marks() {
        // "e" + combining acute accent (é)
        let text = "e\u{0301}";
        let indices = grapheme_indices(text);

        // Should be 1 cluster: e + combining mark
        assert_eq!(indices.len(), 1);
        assert_eq!(indices[0].0, 0); // byte offset of 'e'
        assert_eq!(indices[0].1, 0); // visual column
    }

    #[test]
    fn test_grapheme_indices_zwj_sequence() {
        // Family emoji: man + ZWJ + woman + ZWJ + girl
        // This is 5 code points forming one grapheme cluster
        let text = "👨‍👩‍👧";
        let indices = grapheme_indices(text);

        // Should be 1 grapheme cluster
        assert_eq!(indices.len(), 1);
        assert_eq!(indices[0].0, 0);
        assert_eq!(indices[0].1, 0);
    }

    #[test]
    fn test_grapheme_indices_flag_emoji() {
        // US flag: Regional Indicator U + Regional Indicator S
        let text = "\u{1F1FA}\u{1F1F8}";
        let indices = grapheme_indices(text);

        // Should be 1 cluster representing the flag
        assert_eq!(indices.len(), 1);
        assert_eq!(indices[0].0, 0);
        assert_eq!(indices[0].1, 0); // Flag takes 2 cells, but starts at 0
    }

    #[test]
    fn test_grapheme_indices_mixed() {
        // "Hello 世界 👋"
        let indices = grapheme_indices("Hello 世界 👋");

        // Hello (5) + space (1) + 世界 (2) + space (1) + 👋 (1)
        // Should be: H, e, l, l, o, [space], 世, 界, [space], 👋
        assert_eq!(indices.len(), 10);

        // Check visual column progression
        // H(0) e(1) l(2) l(3) o(4) space(5) 世(6) 界(8) space(10) 👋(11)
        assert_eq!(indices[0].1, 0);
        assert_eq!(indices[1].1, 1);
        assert_eq!(indices[2].1, 2);
        assert_eq!(indices[3].1, 3);
        assert_eq!(indices[4].1, 4);
        assert_eq!(indices[5].1, 5); // space
        assert_eq!(indices[6].1, 6); // 世 (wide)
        assert_eq!(indices[7].1, 8); // 界 (starts at col 8)
        assert_eq!(indices[8].1, 10); // space
        assert_eq!(indices[9].1, 11); // 👋
    }

    #[test]
    fn test_grapheme_indices_empty() {
        let indices = grapheme_indices("");
        assert!(indices.is_empty());
    }

    #[test]
    fn test_grapheme_indices_skin_tone_modifier() {
        // Thumbs up + light skin tone
        let text = "👍\u{1F3FB}";
        let indices = grapheme_indices(text);

        // Should be 1 grapheme cluster
        assert_eq!(indices.len(), 1);
    }

    #[test]
    fn test_grapheme_indices_multiple_flags() {
        // US flag + space + France flag
        let text = "\u{1F1FA}\u{1F1F8} \u{1F1EB}\u{1F1F7}";
        let indices = grapheme_indices(text);

        // Should be: US_flag, space, France_flag
        assert_eq!(indices.len(), 3);
        assert_eq!(indices[0].1, 0); // US flag starts at 0 (takes 2)
        assert_eq!(indices[1].1, 2); // space at 2
        assert_eq!(indices[2].1, 3); // France flag at 3 (takes 2)
    }
}

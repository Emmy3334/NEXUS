//! Lexical analysis for shell command lines (Dragon Book Ch. 3).
//!
//! This slice recognizes only [`TokenKind::Word`]: maximal non-whitespace
//! runs. Operators, quotes, and escapes arrive in later slices.

/// What kind of lexeme a [`Token`] spans.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Word,
}

/// A token stored as a byte span into the source line (no heap copy).
///
/// Call [`Token::lexeme`] with the same `source` that was tokenized to read
/// the text. Spans are invalid after that source buffer is mutated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub start: usize,
    pub end: usize,
}

impl Token {
    /// Borrow the lexeme from `source`.
    ///
    /// # Panics
    ///
    /// Panics if `start..end` is not a valid char boundary range in `source`
    /// (caller must pass the same string that produced this token).
    pub fn lexeme<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start..self.end]
    }
}

/// Tokenize `source` into `tokens`, reusing `tokens`' capacity.
///
/// Clears `tokens` first. One forward scan; no intermediate collections.
pub fn tokenize_into(source: &str, tokens: &mut Vec<Token>) {
    tokens.clear();

    let mut position = 0;
    let source_len = source.len();

    while position < source_len {
        let Some(word_start) = next_non_whitespace(source, position) else {
            break;
        };
        let word_end = next_whitespace_or_end(source, word_start);
        tokens.push(Token {
            kind: TokenKind::Word,
            start: word_start,
            end: word_end,
        });
        position = word_end;
    }
}

fn next_non_whitespace(source: &str, from: usize) -> Option<usize> {
    source[from..]
        .char_indices()
        .find(|(_, ch)| !ch.is_whitespace())
        .map(|(offset, _)| from + offset)
}

fn next_whitespace_or_end(source: &str, from: usize) -> usize {
    source[from..]
        .char_indices()
        .find(|(_, ch)| ch.is_whitespace())
        .map(|(offset, _)| from + offset)
        .unwrap_or(source.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(source: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        tokenize_into(source, &mut tokens);
        tokens
    }

    fn lexemes(source: &str) -> Vec<&str> {
        tokenize(source)
            .iter()
            .map(|token| token.lexeme(source))
            .collect()
    }

    #[test]
    fn empty_and_whitespace_only_yield_no_tokens() {
        assert!(tokenize("").is_empty());
        assert!(tokenize("   \t  ").is_empty());
    }

    #[test]
    fn single_word() {
        assert_eq!(lexemes("ls"), vec!["ls"]);
        assert_eq!(lexemes("  ls  "), vec!["ls"]);
    }

    #[test]
    fn multiple_words_collapse_whitespace() {
        assert_eq!(lexemes("ls -l /tmp"), vec!["ls", "-l", "/tmp"]);
        assert_eq!(
            lexemes("echo   hello\tworld"),
            vec!["echo", "hello", "world"]
        );
    }

    #[test]
    fn tokenize_into_reuses_buffer() {
        let mut tokens = Vec::with_capacity(4);
        tokenize_into("one two", &mut tokens);
        assert_eq!(tokens.len(), 2);
        let capacity_after_first = tokens.capacity();

        tokenize_into("a b c", &mut tokens);
        assert_eq!(
            tokens
                .iter()
                .map(|token| token.lexeme("a b c"))
                .collect::<Vec<_>>(),
            vec!["a", "b", "c"]
        );
        assert!(tokens.capacity() >= capacity_after_first);
    }

    #[test]
    fn words_are_non_empty_and_whitespace_free() {
        let source = "  cmd  -a  ./path  ";
        let tokens = tokenize(source);
        for token in &tokens {
            let lexeme = token.lexeme(source);
            assert!(!lexeme.is_empty());
            assert!(!lexeme.chars().any(char::is_whitespace));
            assert_eq!(token.kind, TokenKind::Word);
        }
    }
}

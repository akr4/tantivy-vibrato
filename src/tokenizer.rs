use std::fs;
use std::io;
use std::io::BufReader;
use std::path;
use std::sync::Arc;
use log::error;
use thiserror::Error;

use tantivy::tokenizer::{BoxTokenStream, Token as TToken, TokenStream, Tokenizer as TTokenizer};
use vibrato::{Dictionary, Tokenizer};

#[derive(Error, Debug)]
pub enum TantivyVibratoError {
    #[error("IO error {0:?}")]
    IOError(#[from] io::Error),
    #[error("vibrate error {0:?}")]
    VibratoError(#[from] vibrato::errors::VibratoError),
}

type Result<T> = std::result::Result<T, TantivyVibratoError>;

#[derive(Clone)]
pub struct VibratoTokenizer {
    tokenizer: Arc<Tokenizer>,
}

impl VibratoTokenizer {
    /// Create a new `VibratoTokenizer`.
    ///
    /// - `dict_path` is the path to the Vibrato dictionary file.
    pub fn new<P: AsRef<path::Path>>(dict_path: P) -> Result<VibratoTokenizer> {
        let file = fs::File::open(&dict_path)?;
        let dict = Dictionary::read(BufReader::new(file))?;
        let tokenizer = Arc::new(Tokenizer::new(dict));

        Ok(VibratoTokenizer { tokenizer })
    }
}

impl TTokenizer for VibratoTokenizer {
    fn token_stream<'a>(&self, text: &'a str) -> BoxTokenStream<'a> {
        let mut worker = self.tokenizer.new_worker();
        worker.reset_sentence(text).unwrap_or_else(|e| {
            error!("Failed to reset sentence: {}", e);
        });
        worker.tokenize();

        let tokens = worker
            .token_iter()
            .map(|t| TToken {
                offset_from: t.range_byte().start,
                offset_to: t.range_byte().end,
                position: t.range_char().start,
                position_length: t.range_char().end - t.range_char().start,
                text: t.surface().to_string(),
            })
            .collect();

        let stream = VibratoTokenStream {
            tokens,
            index: None,
        };

        BoxTokenStream::from(stream)
    }
}

struct VibratoTokenStream {
    tokens: Vec<TToken>,
    index: Option<usize>,
}

impl TokenStream for VibratoTokenStream {
    fn advance(&mut self) -> bool {
        let next_index = self.index.map(|i| i + 1).unwrap_or(0);
        if next_index < self.tokens.len() {
            self.index = Some(next_index);
            true
        } else {
            false
        }
    }

    fn token(&self) -> &TToken {
        &self.tokens[self.index.unwrap()]
    }

    fn token_mut(&mut self) -> &mut TToken {
        &mut self.tokens[self.index.unwrap()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenizer() -> VibratoTokenizer {
        VibratoTokenizer::new("./system.dic")
            .expect("system.dic is required in the project root directory")
    }

    #[test]
    fn test1() {
        let tokenizer = tokenizer();
        let mut stream = tokenizer.token_stream("すもももももももものうち");
        let mut tokens = vec![];
        while let Some(token) = stream.next() {
            tokens.push(token.clone());
        }

        assert_eq!(tokens.len(), 7);
        {
            let token = &tokens[0];
            assert_eq!(token.text, "すもも");
            assert_eq!(token.offset_from, 0);
            assert_eq!(token.offset_to, 9);
            assert_eq!(token.position, 0);
        }
        {
            let token = &tokens[1];
            assert_eq!(token.text, "も");
            assert_eq!(token.offset_from, 9);
            assert_eq!(token.offset_to, 12);
            assert_eq!(token.position, 3);
        }
        {
            let token = &tokens[2];
            assert_eq!(token.text, "もも");
            assert_eq!(token.offset_from, 12);
            assert_eq!(token.offset_to, 18);
            assert_eq!(token.position, 4);
        }
        {
            let token = &tokens[3];
            assert_eq!(token.text, "も");
            assert_eq!(token.offset_from, 18);
            assert_eq!(token.offset_to, 21);
            assert_eq!(token.position, 6);
        }
        {
            let token = &tokens[4];
            assert_eq!(token.text, "もも");
            assert_eq!(token.offset_from, 21);
            assert_eq!(token.offset_to, 27);
            assert_eq!(token.position, 7);
        }
        {
            let token = &tokens[5];
            assert_eq!(token.text, "の");
            assert_eq!(token.offset_from, 27);
            assert_eq!(token.offset_to, 30);
            assert_eq!(token.position, 9);
        }
        {
            let token = &tokens[6];
            assert_eq!(token.text, "うち");
            assert_eq!(token.offset_from, 30);
            assert_eq!(token.offset_to, 36);
            assert_eq!(token.position, 10);
        }
    }

    #[test]
    fn empty() {
        let tokenizer = tokenizer();
        let mut stream = tokenizer.token_stream("");
        let mut tokens = vec![];
        while let Some(token) = stream.next() {
            tokens.push(token.clone());
        }

        assert_eq!(tokens.len(), 0);
    }
}

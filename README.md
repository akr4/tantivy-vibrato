# tantivy-vibrato

A [Tantivy](https://github.com/quickwit-oss/tantivy) tokenizer using [Vibrato](https://github.com/daac-tools/vibrato).

## Usage

```rust
let tokenizer = VibratoTokenizer::new("/path/to/dic")?;
let analyzer = TextAnalyzer::from(tokenizer).filter(LowerCaser);
index.tokenizers().register("lang_ja", analyzer);
```

You need to specify a path to the Vibrato's dictionary file.




# pithpack

Compress codebases into token-lean context bundles for LLMs.

> Pre-release: the design is in [`docs/SDD.md`](docs/SDD.md); vocabulary in [`GLOSSARY.md`](GLOSSARY.md).

## Example

Bundling this repo's own `src/` (output of `pithpack src`; the Bundle goes to stdout, the Report to stderr):

```text
 file                       lang  mode  bytes          tokens           saved  redactions
 src/cli.rs                 rust  none   5.3K → 5.3K    1,429 → 1,429    0.0%           0
 src/config.rs              rust  none  12.6K → 12.6K   3,119 → 3,119    0.0%           0
 …
 src/redact/mod.rs          rust  none  10.2K → 10.2K   2,930 → 2,930    0.0%           7
 …
 ────────────────────────────────────────────────────────────────────────────────────────
 18 files                               74.1K → 74.2K  19,737 → 19,737   0.0%           7
 bundle: 20,318 tokens (o200k_base) · 7 redactions · 534 ms
```

Compression (`--mode strip|skeleton|outline`) isn't implemented yet, so the `saved` column reads 0% today. The design targets roughly 30–70% fewer tokens per file; see [`docs/SDD.md`](docs/SDD.md).

## Install

Once released: `cargo install pithpack`.

## License

Licensed under the [MIT license](LICENSE).

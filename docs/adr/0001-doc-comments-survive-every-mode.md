# Doc Comments survive every Mode

Comments are split into Doc Comments (attached to a declaration) and Free Comments; every Mode removes Free Comments and keeps Doc Comments, unless `--no-docs` is passed. The original brief had Level 0 strip all comments while Level 1 kept docstrings, which is incoherent in languages where a docstring *is* a comment (`///`, `/** */`). Doc Comments describe the API, which is exactly what `skeleton` and `outline` exist to preserve, so dropping them at the cheapest Mode would make `strip` less informative per token than `skeleton`.

## Consequences

Modes are not a strict "each level removes a superset" ladder for comments. Every language adapter must classify comments as Doc or Free, which is per-language work (e.g. Go treats the comment block directly above a declaration as its doc).

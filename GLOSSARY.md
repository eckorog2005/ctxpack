# ctxpack

ctxpack turns source files into a single token-lean bundle for feeding to an LLM, reporting how many tokens it saved.

## Inputs

**Source File**:
One file (or the stdin stream) admitted into a run, after exclusion and ignore rules.
_Avoid_: Document, input, entry

**Plain Text**:
A Source File with no supported language, or stdin without `--lang`; it is redacted and trimmed but never structurally compressed.
_Avoid_: Raw file, unknown file

**Exclusion**:
The rule that a file matching the secret-file list is never read, regardless of ignore files or filters.
_Avoid_: Ignore, skip, blacklist

**Redaction**:
Replacing a secret-looking value inside a Source File with a typed `[REDACTED:<kind>]` marker before anything else sees it.
_Avoid_: Scrubbing, masking, sanitising

## Compression

**Mode**:
How aggressively a Source File is compressed: `strip`, `skeleton` (the default), or `outline`.
_Avoid_: Level (except as the numeric alias), default mode, format

**Strip**:
The Mode that keeps all code and removes only Free Comments and redundant whitespace.
_Avoid_: Format, level 0

**Skeleton**:
The Mode that keeps declarations, signatures, types and Doc Comments, and collapses implementation bodies.
_Avoid_: Default, signatures mode, level 1

**Outline**:
The Mode that keeps only the public API of each Source File.
_Avoid_: Interface mode, level 2

**Doc Comment**:
A comment attached to a declaration that documents it (`///`, `/** */`, Python docstrings, Go leading decl comments); kept in every Mode unless `--no-docs`.
_Avoid_: Docstring (except for Python specifically)

**Free Comment**:
Any comment that is not a Doc Comment; removed in every Mode.
_Avoid_: Inline comment, regular comment

## Output

**Bundle**:
The single artifact a run produces: a File Tree followed by one block per Source File.
_Avoid_: Context file, pack, output

**File Tree**:
The directory listing of the Source Files in a Bundle, placed at its top.
_Avoid_: Directory graph, module tree

**Report**:
The per-file and total original-vs-compressed size and token summary, written to stderr, never into the Bundle.
_Avoid_: Summary table, stats

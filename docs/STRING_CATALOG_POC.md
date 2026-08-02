# Semantic string catalog proof of concept

This branch is a discussion prototype stacked on the DRA EnemyDef asset work in
PR #3456. It demonstrates a way to give strings stable semantic identities and
target-specific encodings without coupling the C build to Rust through FFI.

It deliberately does **not** migrate the retail EnemyDef tables. Those tables
contain blank rows, duplicate visible names, and version-specific differences,
so their IDs need an explicit review rather than being generated from row
numbers or visible text.

## Proposed boundary

The prototype separates four concerns:

1. `tools/sotn_codec/spec.json` describes the font indices, escapes, voice-mark
   composition, and compatibility aliases once.
2. `tools/sotn_codec/generate.py` generates native Rust and Go lookup tables.
   Both tools remain ordinary, portable binaries with no FFI or `cgo` boundary.
3. A YAML catalog maps stable IDs such as `STR_ENEMY_ZOMBIE` to readable text.
   An optional locale or mod overlay replaces text by ID.
4. `sotn-assets build-string-catalog` encodes the selected catalog and emits a
   small C header scoped to the translation unit or table that consumes it.

The generated C keeps the readable text beside each macro:

```c
/* Zombie */
#define STR_ENEMY_ZOMBIE "\x3A\x4F\x4D\x42\x49\x45\xFF"
```

This addresses the visibility concern without relying on the order in which a
target runs its C preprocessor and `sotn_str`. PSX expands includes before the
string processor, while PSP currently runs the string processor before include
expansion. Raw target bytes work in both pipelines.

## Try the complete catalog flow

The examples are synthetic source files; they contain no extracted game data.

```sh
go run ./tools/sotn-assets build-string-catalog \
  tools/sotn_codec/examples/enemy_strings.en.yaml \
  /tmp/enemy_strings.h \
  --platform psx

go run ./tools/sotn-assets build-string-catalog \
  tools/sotn_codec/examples/enemy_strings.en.yaml \
  /tmp/enemy_strings.fr.h \
  --platform psp \
  --overlay tools/sotn_codec/examples/enemy_strings.fr.yaml
```

A source table can then use the stable identity instead of embedding text:

```c
#include "generated/enemy_strings.h"

EnemyDef g_EnemyDefs[] = {
    {STR_ENEMY_ZOMBIE, 10, 5, /* ... */},
};
```

Overlays may be partial, but cannot introduce unknown IDs. Invalid IDs and
characters that the selected target cannot encode are hard errors.

## Shared codec contract

`conformance.json` is executed by both the Go and Rust test suites. It covers
ASCII, PSP accented glyphs, Japanese voice marks, the escaped `月` glyph,
target-specific unsupported glyphs, and invalid inputs. CI also runs the generator in
`--check` mode so an edited spec cannot leave one implementation stale.

The Rust tool is now a library as well as the existing build executable. For a
consumer that must invoke exactly one implementation, it also exposes a
stateless newline-delimited JSON interface:

```sh
printf '%s\n' '{"id":"demo","text":"Aé"}' | \
  cargo run --manifest-path tools/sotn_str/Cargo.toml -- \
  codec encode --platform psp
```

It returns:

```json
{"id":"demo","bytes":"21A0FF"}
```

This batch boundary is suitable for offline tooling and avoids a persistent
helper process. The asset tool uses generated native Go tables directly because
that is simpler, faster, and easier to distribute.

## What this proves, and what remains open

The branch proves compile-time semantic catalogs, locale/mod overlays, strict
target encoding, generated native parity, and translation-unit-scoped headers.
It does not choose production EnemyDef IDs or implement runtime language
selection.

Runtime selection can be a second layer modeled on the PSP code: generated
language tables would share the same semantic ID ordering, and gameplay code
would select a table at runtime. That design needs separate investigation of
memory ownership, font textures, glyph coverage, and which overlays can safely
share storage. The compile-time catalog here is useful whether or not that
runtime layer is eventually adopted.

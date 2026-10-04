# AGENTS.md

Datomic-like database library in Rust (edition 2024), built on persistent Hash Array Mapped Tries
(HAMT). A Cargo workspace: `sky-trie` (the HAMT, `crates/sky-trie`) under `sky-db` (the Datomic layer,
`crates/sky-db`), plus `sky-tui` (a ratatui-kit terminal UI, `crates/sky-tui`) and `universal-hash`
(the hashing primitive, `crates/universal-hash`). Everything is in-memory; there is no file
persistence. No CI, no README.

## Scope & Boundaries

- **Working Directory:** Restrict all file searches, reads, and edits strictly to the current project root and its
  subdirectories (e.g., `src/`, `tests/`, `benches/`).
- **Forbidden Paths:** Do not search or modify external system files, parent directories, or global Cargo
  registries/caches.
- **Allowed Tools:** Limit file exploration to workspace members; ignore hidden directories (`.git`, target build
  artifacts in `target/`).

## Commands

- `cargo test` — all tests across the workspace (unit `#[cfg(test)]` and `tests/` integration), all in-memory.
  Tests are `#[tokio::test]` async.
- `cargo nextest run` — optional; `.config/nextest.toml` is present.
- `cargo clippy` — lint/check (opencode.json configures clippy as the LSP check command).
- `cargo run -p sky-tui` — runs the terminal UI (binary `sky-tui`).
- `rust-analyzer` (CLI) — occasional read-only semantic checks: `rust-analyzer analysis-stats crates/sky-trie`
  and `rust-analyzer diagnostics .`. Flags are unstable; prefer `cargo check` day-to-day.

## Architecture (read top-down in this order)

Layered, each layer building on the one below.

1. `crates/universal-hash` — the single hashing primitive `hash(bytes, level) -> u32`, used via the extern
   prelude (`universal_hash::hash`); not re-exported.
2. `crates/sky-trie` — the HAMT. An in-memory, cursor-based persistent trie. There is no storage trait and no
   file persistence: a `VecBuffer` (a `Vec<Slot>` plus a root index) is the only backing.
   - `objects/` — `SkyTrie` (immutable; `Clone` is a snapshot; edit via `trie.edit(async |t: &mut SkyTrieMut| ...)`)
     and `SkyTrieMut` (a deliberately non-`Clone` edit buffer; `extend(trie)` and `commit()` -> `SkyTrie`).
     `VecBuffer` implements the buffer traits.
   - `traits/` — `Buffer` (`max_index`/`next_index`/`get_root`/`get_base`/`get_subtrie`), `BufferMut`
     (`push_root`/`push_base`/`push_subtrie`), `Query` (`query`/`query_u32`/`query_all`/`query_deep`),
     `KvStream` (`kv_stream`/`map_base_stream`/`u32_stream`), `Snap` (`type Snapshot` + `snapshot`),
     `QueryCursor` (cursor nav: `ascend`/`descend`/`backup`/`restore`/`top_root`), `InsertCursor`
     (`insert_deep`), `Insert` (`insert`/`insert_with_options`, `InsertOption::DeleteOthers`), and the marker
     traits `Trie`/`TrieMut`. `Query`/`Insert`/`KvStream` have blanket impls over `Buffer`/`BufferMut`.
   - `types/` — `MapBase { map: SlotMap, base: BufferIndex }`, `Base { slots: Vec<Slot> }`,
     `Slot::{KeyValue, MapBase, ByteData}`, `KeyValue::{Int, Subtrie, Bytes}` (top-2-bit tag + 30-bit key),
     `TrieValue::{U32, SubTrie, Bytes}`, `SlotMap(u32)` bitmap, `BufferIndex(i32)` (`ZERO`/`NIL`),
     `HashKey`/`DeepKey`, `TrieKey`/`CursorPos`/`Leg`, and `ByteData([u8; 8])` with LEB128-length-prefixed
     byte storage (`push_bytes_to_buffer`/`get_bytes_from_buffer`).
   - `services/` — `base` (`form_kv`/`insert_kv`/`swap_v`, `kick_kv`/`merge_kv`) and `map_base`
     (`one_kv`/`two_kv`, `insert_kv`, `query_value`/`query_keys_values`/`kv_stream`, `query_value_deep`).
3. `crates/sky-db` — the Datomic layer. `lib.rs` exposes `find` and `pull`, keeps `_internal`, `errors`,
   `objects`, `services`, `traits`, `types` private, re-exports them at the root, and `pub use sky_trie as trie`.
   - `objects/` — `Pod { schema: Schema, trie: SkyTrie }` (`Clone` is a snapshot; async
     `Pod::new(db_spec: impl Into<DbSpec>) -> Result<Self, ConnectError>` takes no storage; there is no
     `load`/`close`; `schema()`, `max_tx()`, `ev_stream(attr)`, `get_attribute`, `list_attributes`), `Entity`,
     `Attribute` (`ident`/`cardinality`/`list_binds`), and `Bind(Entity, Val)`.
   - `traits.rs` — `Transact` (`async fn transact(&mut self, ...) -> Result<&mut Self, TransactError>`,
     implemented by `Pod`), `DbQuery` (`find`/`find_val`/`get_val`), and `Find` (associated `type Output`;
     `select`/`where_`/`process`; default `apply` runs `db_trie::find`).
   - `errors.rs` — `ConnectError`, `TransactError` (both wrap `anyhow`), and `QueryError` (currently an empty
     enum).
   - `services/` — `datom::add`/`del`, `db::query`/`ident`/`cardinality`, plus free constructors
     `val`/`dat`/`attr`/`ein`/`ent`.
   - `types/` — `Attr`, `Dat`, `Ein`, `Ent`, `Fill`, `FindResult`, `Val`, `Txid`, `Datom`/`Dir`, and `schema/`
     (`Schema`, `AttrTable`, `AttributeDetails`, `AttrSpec`, `DbSpec`, `Cardinality`).
   - `_internal/` — crate-internal services and types. `services/` holds `datalog` (`Program`, `KnowledgeBase`,
     `Rule`, `Atom`, `Term`, `Var`, `Substitution`), `val_table` (hash-consed `Val`s, returns a `Vid`),
     `db_trie` (storage layout + `with_update`/`find`/`ev_stream`), and `schema` (`save` + `schema_loader`).
     `types/` holds `ent_eid`, `key` (the `KEY_*` prefixes), `max_eid`, `vid`.
   - `find/` — `Find` impls `AllAttrs`, `AllEins`, `AttrsOfEin`, `BindsForAttr`, `EinsWithAttr`, `EntityFills`,
     `ValsInSlot` (several override `apply`; `find/types` is currently empty).
   - `pull/` — `Pull<'a>` (`attrs`/`into_datoms`/`pull`) and `errors::RegisterError`.
   - The query machinery (`find`, datalog, `ev_stream`, `val_table::query`) is generic over
     `T: QueryCursor + KvStream + Snap + Query`, so `SkyTrie` and snapshots both work.
4. `crates/sky-tui` — the ratatui-kit terminal UI. `main.rs` runs `element!(App).fullscreen()`; `routes/` holds
   `app` (with the global `DB_VIEW: Atom<Option<Pod>>`), `home`, and `attribute_binds_table`; `components/`
   holds `loading`; `lines.rs`/`styles.rs` are support modules. When working here, load the `ratatui-kit` skill
   at `.agents/skills/ratatui-kit/SKILL.md`.

## Key gotchas

- **Bit-width constraints are strict.** Trie keys are 30-bit (`TrieKey::MASK = 0x3FFF_FFFF`); the top two bits
  of a `KeyValue` key tag Int/Subtrie/Bytes. `BufferIndex` (non-negative `i32`) is the base handle. `Ein` is a
  non-negative `i32`. The EAVT/AEVT index values still pack `Dir` into bit 28 (`0x1000_0000`) plus a 28-bit
  `Txid`, capping tx ids below 2^28.
- **`BufferIndex::ZERO` is the empty base**; `NIL` is the empty-buffer sentinel. Reads outside `max_index`
  return `Base::empty()`.
- **`Attr` owns a `String`** (attribute idents). Schema attrs must be declared up front: `Pod::new(db_spec)`
  takes `impl Into<DbSpec>` (`[&str; N]`/`Vec<&str>`, `[Attr; N]`, `Vec<Attr>`, `[AttrSpec; N]`). Build an
  `Attr` with `Attr::from("...")`. Entity ids 0–2 are reserved (`Ein::DB_IDENT`, `DB_CARDINALITY`, `DB_MAX`);
  `MaxEid` starts at `Ein::DB_MAX`.
- **`Pod` is snapshot-friendly.** `Pod::clone()` yields an independent snapshot; `Transact::transact(&mut self)`
  edits in place via `SkyTrie::edit`. There is no `close`/re-`load` and no storage handle.
- **`Ent` is `Id(Ein)` or `Temp(String)`.** Temps are assigned `Ein`s per transaction (`ent_eid`); reusing a
  temp ident within one tx targets the same entity, across txs it does not.
- **Errors are minimal.** `ConnectError`/`TransactError` wrap `anyhow`; `QueryError` is an empty enum.
- `universal_hash::hash` (extern prelude) is the hashing primitive; everything keys off it.

## Conventions

- **Line length:** keep every line in this file under 120 characters, except inside special elements such as
  tables.
- Heavily async (`tokio`); most APIs return `impl Future` via `async fn`, often `Result`.
- Symbol-heavy types: `Val` (`U32`/`String`), `TrieValue` (`U32`/`SubTrie`/`Bytes`), `Base`, `Slot`,
  `KeyValue`, `MapBase` (`SlotMap` + `BufferIndex`), `HashKey`/`DeepKey`, plus the db layer's
  `Dat`/`Datom`/`Ent`/`Dir`, `Attr`/`AttrSpec`/`DbSpec`/`Schema`, and `Ein`/`Txid`/`Vid`/`MaxEid`/`EntEid`/
  `AttrEin` — don't confuse the similar names.
- `Dat::Val` vs `Dat::Ent` and `dir` (`Dir::In`/`Dir::Out`, i.e. add/delete) drive query semantics; see
  `crates/sky-db/src/services/datom.rs` (`datom::add` / `datom::del`).

# AGENTS.md

Datomic-like database in Rust (edition 2024), built on persistent Hash Array Mapped Tries (HAMT). It is fully
in-memory — there is no file persistence. The workspace has four crates: `sky-trie` (the HAMT), `sky-db` (the
Datomic layer), `sky-tui` (a terminal UI), and `universal-hash` (the hashing primitive). No CI, no README.

## Conventions

- **Write plainly.** Keep entries concise, readable English with as little inline code as possible. Avoid dense
  type signatures and long lists of symbol names; name at most two or three examples and point to the source for
  the rest.
- **Line length:** keep every line under 120 characters (tables excepted).
- The index write path (`val_table`, `db_trie`, `replay`) is synchronous; the public API (`transact`, `find`,
  `pull`) and the peer/log layer are async (iroh + tokio).
- Names that are easy to confuse: `Ein` (entity id) vs `Vid` (value id) vs `EntEid` (temp-name map); `Dat` (a
  value or entity ref) vs `Datom` (the full entity/attr/value/flag tuple).

## Scope & Boundaries

- **Working directory:** search, read, and edit only inside the project root (e.g. `src/`, `tests/`, `benches/`).
- **Forbidden paths:** do not touch external system files, parent directories, or global Cargo caches.
- **Allowed tools:** explore workspace members only; ignore hidden dirs (`.git`, `target/`).

## Commands

- `cargo test` — all workspace tests (unit `#[cfg(test)]` and `tests/` integration). `sky-trie` tests are
  synchronous; `sky-db` peer/replay tests are async.
- `cargo nextest run` — optional (`.config/nextest.toml` is present).
- `cargo clippy` — lint/check (the LSP check command).
- `cargo run -p sky-tui` — run the terminal UI.

## Architecture

Read top-down; each layer builds on the one below.

1. `crates/universal-hash` — the single hashing primitive, `universal_hash::hash(bytes, level)`.
2. `crates/sky-trie` — the HAMT. Immutable and persistent. `SkyTrie` is the store; `TrieValue` (scalar, nested
   trie, bytes, or nil) is the public value type.
3. `crates/sky-db` — the Datomic layer; re-exports `sky_trie` as `trie`.
    - `objects/` — `Pod`, an immutable index snapshot holding the schema, a `TrieValue`, and the tx order; plus
      the value wrappers `Entity`, `Attribute`, and `Bind`.
    - Transactions and queries come through the `Transact` (on `Peer`), `DbQuery`, and `Find` traits; `find/` is
      a datalog-style engine running over a `Pod`.
    - `errors.rs` — `ConnectError` and `TransactError` wrap `anyhow`; `QueryError` is a placeholder.
    - `services/` — `datom`, `tx`, and `db` modules, plus free constructors (e.g. `val`).
    - `types/` — the data model (entities, attributes, datoms, values) and the schema module.
    - `_internal/` — value hash-consing, the index trie layout, the iroh log and replay, and id bookkeeping.
    - `pull/` — materializes query results into nested maps.
4. `crates/sky-tui` — the ratatui-kit terminal UI; `main.rs` runs `element!(App)`. Load the `ratatui-kit` skill
   at `.agents/skills/ratatui-kit/SKILL.md` before working here.

## Key gotchas

- **Trie keys are non-negative `i32`s** (a 31-bit space). `sky-db` derives entity and value ids by hashing and
  masking to 31 bits (`ID_MASK`, in `_internal/types/key.rs`).
- **`MapBase::empty()` is the empty base** and shares a single static `Arc`.
- **Schema attrs are declared up front** in a `DbSpec`. Build an `Attr` with `Attr::from("...")`. Entity ids
  0–2 are reserved.
- **`Peer` mutates; `Pod` snapshots.** Each transaction produces a new `TrieValue`; `Pod` is a stable read-only
  view from a peer snapshot or replay. There is no `close`/`load` and no storage handle.
- **`Ent` is `Id(Ein)` or `Temp(String)`.** Temps get an `Ein` per transaction; the same temp name targets the
  same entity within one tx but a different one across txs.
- **Errors are minimal:** `ConnectError`/`TransactError` wrap `anyhow`; `QueryError` is an empty enum.
- Everything keys off `universal_hash::hash` in the extern prelude.

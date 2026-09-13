# AGENTS.md

Datomic-like database library written in Rust (edition 2024), built on persistent
Hash Array Mapped Tries (HAMT). A Cargo workspace: `hamt2` (the Datomic-style db, `crates/hamt2`) built on `sky-trie`
(the HAMT, `crates/sky-trie`) and `universal-hash` (the hashing primitive, `crates/universal-hash`); plus
`skybase` (the Leptos web app, which contains its database layer: `skybase::db`). No CI, no README.

## Scope & Boundaries

- **Working Directory:** Restrict all file searches, reads, and edits strictly to the current project root and its
  subdirectories (e.g., `src/`, `tests/`, `benches/`).
- **Forbidden Paths:** Do not search or modify external system files, parent directories, or global Cargo
  registries/caches.
- **Allowed Tools:** Limit file exploration to workspace members; ignore hidden directories (`.git`, target build
  artifacts in `target/`).

## Commands

- `cargo test` — runs all tests across the workspace (unit `#[cfg(test)]` and `tests/` integration). No special filters
  or services required; everything uses in-memory or temp-folder storage.
- Single test: `cargo test <name>` (standard). Tests are `#[tokio::test]` async.
- `cargo leptos build` (run from the workspace root) — builds the `skybase` Leptos web app. `crates/skybase` is the
  Leptos frontend/backend; `skybase::db` is its database layer (a `Db` wrapper over `hamt2` for reading the
  skybase version). `cargo leptos` needs `cargo-leptos` installed; it compiles the `hydrate` feature (wasm) and `ssr`
  feature (native axum server) targets.

## Architecture (read top-down in this order)

Layered, each layer building on the one below. All trie/storage code lives in the separate `sky-trie` crate; `hamt2`
re-exports it as `pub use sky_trie as trie;` in `crates/hamt2/src/lib.rs`, so hamt2 code uses `use
crate::trie::prelude::*` just like before the split.

1. `crates/universal-hash` — the single hashing primitive: `hash(bytes, level) -> u32`. Re-exported as
   `hamt2::universal_hash`.
2. `crates/sky-trie` — the HAMT.
   - `trie_storage/` — persistence abstraction. Traits `ReadTrieStorage: Sync`
     (`read`/`max_id`/`read_root`/`get_root`, errors `TrieStorageReadError`) and `ReadWriteTrieStorage`
     (`next_id`/`append`/`write_root`/`to_readonly` + `type ReadOnly`, errors `TrieStorageWriteError`).
     Implementations: `mem::MemTrieStorage` (a `Vec<SlotBase>` seeded with the empty base at index 0;
     `MemReadStorage` snapshots) and `file::FileTrieStorage` (postcard-encoded base files in two-level subfolders
     under `<folder>/bases/`, with `max_id` and `root` files in the folder root; `FileReadStorage` snapshots).
   - `error.rs` — the trie's own error layer: `TrieQueryError` (wraps `TrieStorageReadError`) and `TrieWriteError`
     (`ExpectedMapBaseAtKey`, wraps `TrieQueryError`). Nothing in the trie produces hamt2's db-level errors.
   - `types/` — `MapBase { map: SlotMap, base: SlotBaseId }` (a node: bases are read from storage, never inline
     slots), `SlotBase { slots: Vec<Slot> }`, `Slot::KeyValue(i32, TrieValue) | MapBase(MapBase)`,
     `TrieValue::U32(u32) | SubTrie(MapBase)`, plus `HashKey`/`HashKeyPath`. `SlotBaseId(0)` is the reserved empty
     base.
   - `Trie<S: ReadWriteTrieStorage>` — the persistent map: `connect(storage)` (`-> TrieStorageReadError`) loads the
     persisted root, mutations (`insert`, `deep_insert`) consume and return a new `Trie` (`-> TrieWriteError`),
     `.commit()` (`-> TrieStorageWriteError`) writes the root, `.view()` gives a borrowed `TrieRef`. Queries live on
     the `TrieQuery<S: ReadTrieStorage>` trait (`query_value`, `query_keys_values`, `deep_query_value`, `u32_stream`,
     `subtrie_stream`, `to_subtrie_from_value`; `-> TrieQueryError`), implemented by `Trie`, `TrieRef<'a, S>`, and
     `TrieReader<S>` (read-only, connects over `ReadTrieStorage` only, e.g. `storage.to_readonly()`). The `prelude`
     re-exports all of the above.
3. `crates/hamt2` — the Datomic layer plus error glue.
   - `src/error/` — `QueryError` (embeds `TrieQueryError` as `QueryError::Trie`), `TransactError` (embeds
     `TrieStorageRead`/`TrieStorageWrite`/`TrieWriteError` and adds db-level variants like `NoSpaceInValueTable`),
     and `LoadError` (`TrieStorageRead`, `UnknownAttr`). Re-exported at the crate root (`pub use error::*`).
   - `src/db/` — `Datom` (`ent`/`attr`/`dat`/`dir`), schema, and queries. `Db<S>` wraps `schema` + `trie: Trie<S>`.
     `DbReader<S: ReadTrieStorage>` is a read-only snapshot: `DbReader::load(db)` *borrows* a `Db<T>` and builds an
     independent snapshot reader over `T::ReadOnly` (bound via `T: ReadWriteTrieStorage<ReadOnly = S>`, so `T` stays
     inferable from the `Db`); the `Db` remains usable, and writes after `load` are invisible to the reader. It
     implements `DbQuery`. The read-only query machinery (`src/find/` `Find` impls,
     `db/datalog::{Program, KnowledgeBase, atom, rule}`, `db/component/db_trie::{find, ev_stream}`,
     `db/component/val_table::query`) is generic over `T: TrieQuery<S>, S: ReadTrieStorage`, so it works for `Trie`,
     `TrieRef`, and `TrieReader` alike; transitive helpers returning `TrieRef<'a, S>` need `S: 'a`.
     `DbHandle<S: ReadWriteTrieStorage + Send + Sync + 'static>` (`db/handle.rs`) owns the `Db` in a worker task;
     `transact` sends datoms over an mpsc channel and `to_reader()` round-trips a `DbReader<S::ReadOnly>` snapshot
     through it.

## Skybase frontend layout

Within `crates/skybase/src`:

- `routes/` — route-level/page components, one module per route (`routes/home.rs`). Pages own route wiring and data
  fetching (via `#[server]` calls in `api/` or resources); they are mounted in `<Route>`s in `app.rs`.
- `components/` — reusable presentational components with no data logic; they receive everything as props. If a
  component fetches or depends on `skybase::db`/`hamt2` data directly, it belongs in `routes/` (or its data should be loaded
  in a `route` and passed down).
- `api/` — all `#[server]` functions (isomorphic: the same definition compiles to a client stub under `hydrate` and a
  server impl under `ssr`). Keep them out of components.
- `state/` — app-wide shared context: signals/resources and the types provided with `provide_context` / read with
  `expect_context`.
- `app.rs` — the `App` root (Router + `shell`); `server.rs` — the axum `serve()` glue (ssr-only).

## Key gotchas

- **Bit-width constraints are strict.** Trie keys are 31-bit (non-negative `i32`); negative keys panic with
  `assertion failed: value >= 0`. Trie values are `TrieValue::U32(u32)` (32-bit). `Ein` (entity id) and `SlotBaseId`
  are non-negative `i32`s.
- **`SlotBaseId(0)` is the empty base.** It is never stored; every storage returns an empty `SlotBase` for it and
  appends start at id 1.
- **`Attr` is `&'static str`** (attribute idents), not an integer. Schema attributes must be declared up front:
  `Db::new(storage, [attrs])` / `Db::load(storage, [attrs])` enumerate every `Attr` used. Loading with an undeclared
  attr fails with `LoadError::UnknownAttr`.
- **`Db` is immutable-value / consumed-ownership.** `Db::transact(...)` consumes `self` and returns a new `Db`. Get the
  underlying storage back with `db.close()` before re-`load`ing.
- **Generic bounds are pervasive.** Any struct/fn mentioning `Trie<S>` or `Db<S>` needs `S: ReadWriteTrieStorage`.
  Read-only sub-tries are borrowed views (`TrieRef<'a, S>` from `trie.view()` / `to_subtrie_from_value`); nothing
  requires `S: Clone`.
- **Query methods live on the `TrieQuery<S: ReadTrieStorage>` trait** (`query_value`, `query_keys_values`,
  `deep_query_value`, `u32_stream`, `subtrie_stream`, `to_subtrie_from_value`), implemented by `Trie`, `TrieRef`, and
  `TrieReader`. Only `root()` / `storage()` are required. Calling a query method needs `TrieQuery` in scope (it comes
  with `use crate::trie::prelude::*`); mutation methods (`insert`, `deep_insert`, `commit`) stay inherent on `Trie`.
  `TrieReader` connects to a `ReadTrieStorage` only (e.g. `storage.to_readonly()`).
- **Errors are layered.** The trie crate only produces `TrieQueryError` / `TrieWriteError` (plus
  `TrieStorageReadError`/`TrieStorageWriteError`); hamt2's `QueryError`/`TransactError`/`LoadError` embed them as
  `QueryError::Trie`, `TransactError::Trie`, `LoadError::TrieStorageRead`, so `?` chains across crates work through
  `From` impls (`use crate::trie::prelude::*` brings the trie error types in scope).
- **`Ent` is either `Id(Ein)` or `Temp(&'static str)`.** Temp entities get auto-assigned `Ein`s at transact time (see
  `src/db/component/ent_eid.rs`). Reusing the same temp ident in a tx rewrites the same entity, whereas separate txns
  create separate entities.
- `universal_hash::hash` (crate `universal-hash`, re-exported as `hamt2::universal_hash`) is the hashing primitive;
  everything keys off it.

## Conventions

- Heavily async (`tokio`); most APIs return `impl Future` via `async fn` with `Result`.
- Symbol-heavy internal types: `Val` (db user value, `U32`/`String`), `TrieValue` (`U32`/`SubTrie(MapBase)`),
  `SlotBase` (a trie node's `Vec<Slot>`), `Slot` (`KeyValue`/`MapBase`), `MapBase` (`SlotMap` + `SlotBaseId`),
  `HashKey`/`HashKeyPath` — don't confuse them despite similar names.
- `Dat::Val` vs `Dat::Ent` and `dir` (`Dir::In`/`Dir::Out`, i.e. add/delete) drive query semantics; see
  `src/db/core/datom/mod.rs` (`datom::add` / `datom::del`).

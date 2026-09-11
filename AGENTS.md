# AGENTS.md

Datomic-like database library written in Rust (edition 2024), built on persistent
Hash Array Mapped Tries (HAMT). A Cargo workspace: library crate `hamt2` (in `crates/hamt2`). No CI, no README.

## Commands

- `cargo test` — runs all tests across the workspace (unit `#[cfg(test)]` and `tests/` integration). No special filters or services required; everything uses in-memory or temp-folder storage.
- Single test: `cargo test <name>` (standard). Tests are `#[tokio::test]` async.
- `cargo leptos build` (run from the workspace root) — builds the `skybase` Leptos web app. `crates/skybase` is the Leptos frontend/backend; `crates/skydb` is its database layer (a `DbViewer`/`Db` wrapper over `hamt2` for reading the skybase version). `cargo leptos` needs `cargo-leptos` installed; it compiles the `hydrate` feature (wasm) and `ssr` feature (native axum server) targets.

## Architecture (read top-down in this order)

Layered, each layer building on the one below:

1. `src/trie/base_storage/` — persistence abstraction for trie `Base`s. Traits `BaseStorageRead`/`BaseStorageReadWrite` (`read`/`max_id`/`read_root`; `next_id`/`append`/`write_root`). Implementations: `mem::MemBaseStorage` (a `Vec<Base>` seeded with the empty base at index 0) and `file::FileBaseStorage` (postcard-encoded base files in two-level subfolders under `<folder>/bases/`, with `max_id` and `root` files in the folder root).
2. `src/trie/` — the HAMT. `TrieMapBase { map: TrieMap, base: BaseId }` is a node: the `base` field is a `BaseId` into a storage, never inline slots. `BaseId(0)` is the reserved empty base. `Trie<S: BaseStorageReadWrite + Clone>` is the persistent map: `connect(storage)` loads the persisted root, mutations consume and return a new `Trie`, `.commit()` writes the root to the storage.
3. `src/db/` — the Datomic layer over `Trie`: `Datom` (`ent`/`attr`/`dat`/`dir`), schema, and queries (`find`/`pull`). `Db<S>` wraps `schema` + `trie: Trie<S>`.

## Skybase frontend layout

Within `crates/skybase/src`:

- `routes/` — route-level/page components, one module per route (`routes/home.rs`). Pages own route wiring and data fetching (via `#[server]` calls in `api/` or resources); they are mounted in `<Route>`s in `app.rs`.
- `components/` — reusable presentational components with no data logic; they receive everything as props. If a component fetches or depends on `skydb`/`hamt2` data directly, it belongs in `routes/` (or its data should be loaded in a `route` and passed down).
- `api/` — all `#[server]` functions (isomorphic: the same definition compiles to a client stub under `hydrate` and a server impl under `ssr`). Keep them out of components.
- `state/` — app-wide shared context: signals/resources and the types provided with `provide_context` / read with `expect_context`.
- `app.rs` — the `App` root (Router + `shell`); `server.rs` — the axum `serve()` glue (ssr-only).

## Key gotchas

- **Bit-width constraints are strict.** Trie keys are 31-bit (non-negative `i32`); negative keys panic with `assertion failed: value >= 0`. Trie values are `u32` (32-bit). `Ein` (entity id) is a non-negative `i32`, `BaseId` is a non-negative `i32`.
- **`BaseId(0)` is the empty base.** It is never stored; every storage returns an empty `Base` for it and appends start at id 1.
- **`Attr` is `&'static str`** (attribute idents), not an integer. Schema attributes must be declared up front: `Db::new(storage, [attrs])` / `Db::load(storage, [attrs])` enumerate every `Attr` used. Loading with an undeclared attr fails with `LoadError::UnknownAttr`.
- **`Db` is immutable-value / consumed-ownership.** `Db::transact(...)` consumes `self` and returns a new `Db`. Get the underlying storage back with `db.close()` before re-`load`ing.
- **Generic bounds are pervasive.** Any struct/fn mentioning `Trie<S>` or `Db<S>` needs `S: BaseStorageReadWrite`. Read-only sub-tries are borrowed views (`TrieRef<'a, S>` from `trie.view()` / `to_subtrie_from_value`); nothing requires `S: Clone`.
- **Query methods live on the `TrieQuery<S: BaseStorageRead>` trait** (`query_value`, `query_keys_values`, `deep_query_value`, `u32_stream`, `subtrie_stream`, `to_subtrie_from_value`), implemented by `Trie`, `TrieRef`, and `ReadTrie`. Only `root()` / `storage()` are required. Calling a query method needs `TrieQuery` in scope (`use crate::trie::TrieQuery`); mutation methods (`insert`, `deep_insert`, `commit`) stay inherent on `Trie`. `ReadTrie` connects to a `BaseStorageRead` only (e.g. `storage.to_readonly()`).
- **`Ent` is either `Id(Ein)` or `Temp(&'static str)`.** Temp entities get auto-assigned `Ein`s at transact time (see `src/db/component/ent_eid.rs`). Reusing the same temp ident in a tx rewrites the same entity, whereas separate txns create separate entities.
- `hash::universal` is the hashing primitive; everything keys off it.

## Conventions

- Heavily async (`tokio`); most APIs return `impl Future` via `async fn` with `Result`.
- Symbol-heavy internal types: `Val` (user value, `U32`/`String`), `MemValue` (`U32`/`MapBase(TrieMapBase)`), `Base` (a trie node's `Vec<MemSlot>`), `MemSlot` (`KeyValue`/`MapBase`) — don't confuse them despite similar names.
- `Dat::Val` vs `Dat::Ent` and `dir` (`Dir::In`/`Dir::Out`, i.e. add/delete) drive query semantics; see `src/db/core/datom/mod.rs` (`datom::add` / `datom::del`).

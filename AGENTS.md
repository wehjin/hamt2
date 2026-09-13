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
3. `crates/hamt2` — the Datomic layer plus error glue. Public modules in `src/lib.rs`: `datom`, `db`, `find`,
   `handle`, `pull`, `query`, `reader`, `transact`, `types` (plus `pub(crate) crate_services`), with
   `pub use sky_trie as trie; pub use universal_hash; pub use error::*;`.
   - `src/datom/` — `Datom { ent, attr, dat, dir }` with the `add`/`del` constructors; `dat::Dat` (`Val(Val)`/`Ent(Ent)`)
     and `ent::Ent` (`Id(Ein)`/`Temp(&'static str)`).
   - `src/types/` — user-facing value types: `Attr(&'static str)` (idents), `AttrName(String)`, `Ein(pub i32)`
     (non-negative; 0–2 reserved: `DB_IDENT`, `DB_CARDINALITY`, `DB_MAX`), `Txid(u32)` (`SETUP` = 0, `FLOOR` = 1),
     `Dir` (`In` = add / `Out` = delete), `Val` (`U32(u32)`/`String`). `src/types/schema/` — `Schema` (newtype over
     `AttrTable` via `Deref`), `AttrTable` (`HashMap<Attr, Attribute>`: `Index<Attr>`, always seeded with the
     `db/ident` and `db/cardinality` starter attributes), `Attribute { ein, spec: AttrSpec }`,
     `AttrSpec { attr, cardinality }`, `Cardinality` (`One`/`Many`), `attr_loader::AttributeLoader` (a `Find` impl
     used by `Db::load` to read attrs back out of the trie).
   - `src/db/` — `db/mod.rs` builds `Db<S>` (`schema: Schema` + `trie: Trie<S>`). Construction:
     `Db::new(storage, db_spec: impl Into<DbSpec>)` (`-> TransactError`; `DbSpec` is `From<[Attr; N]>`,
     `From<Vec<Attr>>`, `From<[AttrSpec; N]>`) assigns fresh `Ein`s from `MaxEid`, saves the schema under
     `Txid::SETUP`, resets max tx to `Txid::FLOOR`, commits; `Db::load(storage, attrs: impl AsRef<[Attr]>)`
     (`-> LoadError`) reopens against `Schema::load` (undeclared attr => `LoadError::UnknownAttr`); `close()` returns
     the storage; inherent `to_reader()` and `max_tx()`. Also the reserved attr idents `db/query` (`QUERY`),
     `db/ident` (`IDENT`), `db/cardinality` (`CARDINALITY`).
     `src/db/db_trie.rs` — storage layout plus the datalog bridge. Trie keys are `[i32-prefix, ...]`, prefixes per
     `src/db/types/key.rs`: `KEY_MAX_TXID` (0), `KEY_EAVT` (1), `KEY_AEVT` (2), `KEY_MAX_EID` (3), `KEY_VAL_TABLE` (4).
     Datoms live at `[KEY_EAVT, eid, aid, vid]` and `[KEY_AEVT, aid, eid, vid]`; the value is a packed `u32`
     (`Dir` in bit 28, 28-bit `Txid`, see `db_trie::Value`). `with_update` (val-table insert + `deep_insert` into
     both indexes, `replace_tail` when `Cardinality::One`), `set_max_tx`, `find` (runs a datalog query),
     `ev_stream(attr)` (`(eid, Val)` stream), `list_entities`, `list_entity_attributes`, `AttrEin` (an `Ein` naming
     an attribute).
     `src/db/types/` — `ent_eid::EntEid` (temp-ident -> `Ein` map per tx), `key`, `max_eid::MaxEid`
     (`read`/`take`/`write`; starts at `Ein::DB_MAX` when unset), `vid::Vid` (val-table id).
   - `src/crate_services/` (`pub(crate)`) — internal services: `datalog/` (the query engine: `Program` (range-restricted
     check + naive fixpoint over `KnowledgeBase::step`), `KnowledgeBase`, `Rule`, `Atom { attr, terms }`,
     `Term` (`Var`/`Val`), `Var(&'static str)`, `Substitution`); `val_table` (hash-consed values: bytes packed into
     `u32` subkeys under `[KEY_VAL_TABLE, vid]`, insert returns the `Vid`, equal values dedupe); `u32/` (`Stream`
     (bytes -> (`u32_subkey`, u32) iterator) / `Read` (per-u32 trie reads -> bytes)).
   - `src/error/` — `QueryError` (`Anyhow`/`SerdeJson`/`Io`/`Utf8`/`SerdeError`/`Trie(TrieQueryError)`, doubles as a
     `serde::de::Error`), `TransactError` (`Anyhow`/`SerdeJson`/`Query(QueryError)`/`TrieStorageRead`/
     `TrieStorageWrite`/`Trie(TrieWriteError)`/`HighBitInValue`/`NoSpaceInValueTable`), `LoadError` (`QueryError`/
     `TrieStorageRead`/`UnknownAttr`). Re-exported at the crate root.
   - `src/find/` — the `Find` trait (`select()` + `where_() -> Vec<Atom>` + `process(FindResult)`; default `apply`
     runs `db_trie::find`; `FindResult` = `Vec<HashMap<String, Val>>`). Impls: `all_eins`, `attr_with_name`,
     `attrs_of_ein`, `binds_for_attr`, `eins_with_attr`, `vals_in_slot`; each compiles to a datalog `rule` headed by
     `db/query`.
   - `src/query.rs` — `DbQuery` trait (`find`, `get`, `find_val`, `get_val`), implemented by `Db` and `DbReader`.
   - `src/reader.rs` — `DbReader<S: ReadTrieStorage>`: `DbReader::load(db)` *borrows* a `Db<T>` and builds an
     independent snapshot via `T::ReadOnly` (bound `T: ReadWriteTrieStorage<ReadOnly = S>`, so `T` stays inferable
     from the `Db`); the `Db` remains usable, and writes after `load` are invisible to the reader.
   - `src/transact.rs` — `Db::transact(self, datoms)` consumes and returns a new `Db`: resolves temps via `EntEid`,
     inserts values in the val table, `with_update` per datom, `set_max_tx(tx + 1)`, writes `MaxEid`, commits.
   - `src/handle.rs` — `DbHandle<S: ReadWriteTrieStorage + Send + Sync + 'static>` owns the `Db` in a worker task;
     `transact` sends datoms over an mpsc channel, `to_reader()` returns a `DbReader<S::ReadOnly>` snapshot; errors
     as `HandleError` (`TaskClosed`/`TaskFailed`/`TransactFailed`/`LoadFailed`).
   - `src/pull/` — `Pull` trait (`Serialize + Deserialize`; `attrs()` / `into_datoms()` / `pull(&Db, eid)`),
     `errors::RegisterError::DuplicateAttr`, tests in `pull/tests/`.
   - The read-only query machinery (`find`, datalog, `ev_stream`, `val_table::query`) is generic over
     `T: TrieQuery<S>, S: ReadTrieStorage`, so it works for `Trie`, `TrieRef`, and `TrieReader` alike; transitive
     helpers returning `TrieRef<'a, S>` need `S: 'a`. Unit tests are `#[cfg(test)]` beside the code (e.g. in
     `query.rs`, `find/mod.rs`, `crate_services/datalog/mod.rs`, `crate_services/val_table.rs`); integration tests in
     `crates/hamt2/tests/` (cardinality, db_reader, file_db, handle, mem_db, multiple_entities).

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
  are non-negative `i32`s. `TrieValue`s stored in the EAVT/AEVT indexes pack `Dir` into bit 28 (`0x1000_0000`) and
  a 28-bit `Txid` into the low bits, so tx ids are capped below 2^28.
- **`SlotBaseId(0)` is the empty base.** It is never stored; every storage returns an empty `SlotBase` for it and
  appends start at id 1.
- **`Attr` is `&'static str`** (attribute idents), not an integer. Schema attributes must be declared up front:
  `Db::new(storage, db_spec)` takes `impl Into<DbSpec>` (`[Attr; N]`, `Vec<Attr>`, or `[AttrSpec; N]` for
  cardinality), and `Db::load(storage, attrs)` takes `impl AsRef<[Attr]>`; every `Attr` used must be enumerated.
  Loading with an undeclared attr fails with `LoadError::UnknownAttr`. Entity ids 0–2 are reserved (`Ein::DB_IDENT`,
  `Ein::DB_CARDINALITY`, `Ein::DB_MAX`); fresh eins (attributes at `Db::new`, temps at `transact`) are handed out by
  `MaxEid`, which starts at `Ein::DB_MAX`.
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
  `TrieStorageReadError`/`TrieStorageWriteError`); hamt2's `QueryError`/`TransactError`/`LoadError` embed them via
  `QueryError::Trie`, `TransactError::TrieStorageRead`/`TransactError::TrieStorageWrite`/`TransactError::Trie`,
  and `LoadError::TrieStorageRead`, so `?` chains across crates work through `From` impls (`use crate::trie::prelude::*`
  brings the trie error types in scope). `TransactError` also embeds `QueryError` as `TransactError::Query`.
- **`Ent` is either `Id(Ein)` or `Temp(&'static str)`.** Temp entities get auto-assigned `Ein`s at transact time (see
  `src/db/types/ent_eid.rs`). Reusing the same temp ident in a tx rewrites the same entity, whereas separate txns
  create separate entities.
- `universal_hash::hash` (crate `universal-hash`, re-exported as `hamt2::universal_hash`) is the hashing primitive;
  everything keys off it.

## Conventions

- Heavily async (`tokio`); most APIs return `impl Future` via `async fn` with `Result`.
- Symbol-heavy internal types: `Val` (db user value, `U32`/`String`), `TrieValue` (`U32`/`SubTrie(MapBase)`),
  `SlotBase` (a trie node's `Vec<Slot>`), `Slot` (`KeyValue`/`MapBase`), `MapBase` (`SlotMap` + `SlotBaseId`),
  `HashKey`/`HashKeyPath`, plus the db layer's `Dat`/`Datom`/`Ent`/`Dir`, `Attr`/`AttrName`/`AttrSpec`/`DbSpec`/`Schema`,
  `Ein`/`Txid`/`Vid`/`MaxEid`/`EntEid`/`AttrEin` — don't confuse the similar names despite the overlap.
- `Dat::Val` vs `Dat::Ent` and `dir` (`Dir::In`/`Dir::Out`, i.e. add/delete) drive query semantics; see
  `src/datom/mod.rs` (`datom::add` / `datom::del`).

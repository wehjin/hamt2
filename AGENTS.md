# AGENTS.md

Datomic-like database library written in Rust (edition 2024), built on persistent
Hash Array Mapped Tries (HAMT). A Cargo workspace: `sky-db` (the Datomic-style db, `crates/sky-db`) built on `sky-trie`
(the HAMT, `crates/sky-trie`) and `universal-hash` (the hashing primitive, `crates/universal-hash`); plus
`sky-server` (socket server, `crates/sky-server`) and
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
  Leptos frontend/backend; `skybase::db` is its database layer (a `Db` wrapper over `sky-db` for reading the
  skybase version). `cargo leptos` needs `cargo-leptos` installed; it compiles the `hydrate` feature (wasm) and `ssr`
  feature (native axum server) targets.
- `rust-analyzer` (CLI) — available for occasional read-only semantic checks: `rust-analyzer analysis-stats
  crates/sky-trie` (full semantic analysis + stats, independent of `cargo check`) and `rust-analyzer diagnostics .`
  (LSP-style diagnostic dump). Not an interactive query tool — no per-position hover/types; subcommand flags are
  unstable, so prefer `cargo check` for day-to-day diagnostics and use this to cross-check or reproduce LSP behavior.

## Architecture (read top-down in this order)

Layered, each layer building on the one below. All trie/storage code lives in the separate `sky-trie` crate; `sky-db`
imports the trie privately (`use sky_trie as trie;` in `crates/sky-db/src/lib.rs`), so sky-db code uses `use
crate::trie::prelude::*` internally. sky-db does not re-export any storage types: callers name
`sky_types::storage::*` directly (`MemStorage`/`FileStorage`, `ReadStorage`/`ReadWriteStorage`,
`ReadStorageError`/`WriteStorageError`), which means storage-consuming crates (skybase, sky-server) depend on
`sky-trie` themselves.

1. `crates/universal-hash` — the single hashing primitive: `hash(bytes, level) -> u32`. A direct sky-db/sky-trie
   dependency, referenced via the extern prelude (`universal_hash::hash`); not re-exported.
2. `crates/sky-trie` — the HAMT.
   - `storage/` — persistence abstraction. `ReadStorage: Sync`
      (`read`/`max_id`/`read_root`/`snapshot`/`get_head` + `type Snapshot`, errors `ReadStorageError` (Io/Decode);
      `read` returns the empty base for `SlotBaseId::ZERO` and panics on unwritten ids (writer storages) or ids
      beyond a snapshot's captured `max_id`; `max_id`
      returns the highest id and counts the reserved empty base, so an empty storage returns `SlotBaseId::ZERO`;
      `read_root` (sync, infallible) returns the committed root, defaulting to `MapBase::empty()` when none is
      committed — every implementation holds the root in memory, even file storage, which reads it once at
      `FileStorage::load`; `get_head` (sync) = `StorageHead { max_id, root }`; every
      storage is snapshottable — writers capture their read-only snapshot, read-only types use `Self` via `Clone`)
      and `ReadWriteStorage`
      (`next_id`/`append`/`write_root`, errors `WriteStorageError` (Io/Encode)).
      `ReadStorage: TrieReadPolicy + Sync` — the link config comes from `TrieReadPolicy::Config` (a supertrait
      item, not a second declaration), and `Snapshot: ReadStorage + TrieReadPolicy<Config = Self::Config>`.
      Implementations: `mem::MemStorage` (a `Vec<SlotBase>` seeded with the empty base at index 0;
      `MemReadStorage` snapshots) and `file::FileStorage` (postcard-encoded base files in two-level subfolders
      under `<folder>/bases/`, with `max_id` and `root` files in the folder root; `FileReadStorage` snapshots).
    - `error.rs` — the trie's own error layer lives in `sky_types::trie`: `TrieQueryError` (`SystemError`) and
      `TrieInsertError` (`ExpectedMapBaseAtKey`, wraps `TrieQueryError`). Nothing in the trie produces sky-db's
      db-level errors.
   - `types/` — `TrieConfig` (`type HandleType: Clone + Eq + PartialEq + Default + Debug`; the trie types are
     generic over a config `C: TrieConfig`, not the raw handle type) and the handle-link config
     `HandleTrieConfig { HandleType = SlotBaseId }`. `MapBase<C> { map: SlotMap, base: C::HandleType }` (a node:
     bases are read from storage, never inline slots), `SlotBase<C> { slots: Vec<Slot<C>> }`,
     `Slot<C>::KeyValue(i32, TrieValue<C>) | MapBase(MapBase<C>)`,
     `TrieValue<C>::U32(u32) | SubTrie(MapBase<C>)`, plus `HashKey`/`DeepKey`. `SlotBaseId::ZERO` is the reserved
     empty base. Serde derives on the config-generic types carry explicit `#[serde(bound(.. = "C::HandleType: ..
     'de"))]` attributes (configs are marker structs; concrete `HandleTrieConfig` serializes over `SlotBaseId`).
    - `Trie<S: ReadWriteStorage>` — the persistent map: `connect(storage)` (sync, infallible) loads the
      persisted root, mutations (`insert`, `deep_insert`) consume and return a new `Trie` (`-> TrieInsertError`),
      `.commit()` (`-> WriteStorageError`) writes the root, `.view()` gives a `TrieReader<S::Snapshot>` snapshot.
     Queries live on
     the config-generic `TrieQuery<C: TrieConfig>` trait (defined in `sky-types` under `sky_types::trie`,
     alongside `TrieValue`; re-exported by the prelude; `root`, `query_value`, `query_keys_values`,
     `deep_query_value`,
     `u32_stream`,
     `subtrie_stream`, `to_subtrie_from_value`, plus `type Subtrie: TrieQuery<C>`; `-> TrieQueryError`) — all
     methods
     required, no defaults, no storage types mentioned. `StorageTrieQuery<S: ReadStorage>` supertrait adds
     `storage()`; `Trie` and `TrieReader<S>` implement `TrieQuery<S::Config>` (`type Subtrie =
     TrieReader<S::Snapshot>`; root + all query methods come from the direct `TrieQuery` impls in
     `storage_trie_query.rs`, which delegate to the free fns in `crate_services/map_base`). The read/write
     policies `TrieReadPolicy { type Config: TrieConfig; read_base(&self, id: C::HandleType) -> SlotBase<C> }`
     and `TrieWritePolicy: TrieReadPolicy` (with `kick_kv`/`merge_kv` default methods) live in
     `sky_types::trie::traits`; every storage gets its policy impl (the four sky-types storages via a macro,
     downstream storages implement `TrieReadPolicy`/`TrieWritePolicy` for themselves with
     `Config = HandleTrieConfig`). `subtrie_stream()` and `to_subtrie_from_value()` yield `Self::Subtrie`, so
     callers never name `TrieReader`. The `prelude` re-exports all of the above.
3. `crates/sky-db` — the Datomic layer plus error glue. Public modules in `src/lib.rs`: `db`, `find`,
   `handle`, `pull`, `query`, `reader`, `transact`, `types` (plus `pub(crate) crate_services`), with
   `pub use error::*;` at the root. No storage re-exports.
   - `src/types/` — user-facing value types: `Attr(String)` (idents; construct with `Attr::from("...")` —
     a plain `Attr("...")` with a literal no longer compiles), `Ein(pub i32)`
     (non-negative; 0–2 reserved: `DB_IDENT`, `DB_CARDINALITY`, `DB_MAX`), `Txid(u32)` (`SETUP` = 0, `FLOOR` = 1),
     `Dir` (`In` = add / `Out` = delete), `Val` (`U32(u32)`/`String`); the datom machinery: `dat::Dat`
     (`Val(Val)`/`Ent(Ent)`), `ent::Ent` (`Id(Ein)`/`Temp(String)`), and `datom::{add, del}` constructors
     building `datom::Datom { ent, attr, dat, dir }` (`Datom` is Serialize/Deserialize; re-exported at the `types`
     root, and from there
     `pub use`d out of `db/mod.rs`, so `sky_db::db::Datom`/`Dat`/`Ent` all resolve; `datom::add`/`datom::del` come
     from `sky_db::types::datom`). `src/types/schema/` — `Schema` (newtype over
     `AttrTable` via `Deref`), `AttrTable` (`HashMap<Attr, Attribute>` keyed by ident plus an `Ein -> Attr`
     reverse index for O(1) `find_attr(ein)`; `Index<Attr>`, always seeded with the
     `db/ident` and `db/cardinality` starter attributes), `Attribute { ein, spec: AttrSpec }`,
      `AttrSpec { attr, cardinality }`, `Cardinality` (`One`/`Many`), `schema_loader::SchemaLoader` (a `Find` impl
      in `src/schema/`, used by `Db::load` to read all attrs back out of the trie).
   - `src/db/` — `db/mod.rs` builds `Db<S>` (`schema: Schema` + `trie: Trie<S>`). Construction:
     `Db::new(storage, db_spec: impl Into<DbSpec>)` (`-> TransactError`; `DbSpec` is `From<[&str; N]>`,
     `From<Vec<&str>>`, `From<[Attr; N]>`, `From<Vec<Attr>>`, `From<[AttrSpec; N]>`) assigns fresh `Ein`s from
     `MaxEid`, saves the schema under
      `Txid::SETUP`, resets max tx to `Txid::FLOOR`, commits; `Db::load(storage: S)` (async, infallible —
      `-> Self`) rebuilds the full schema via `schema::load` (every attr's `db/ident`/`db/cardinality`
      datoms read out of the trie); `close()` returns
     the storage; inherent `to_reader()` and `max_tx()`. Also the reserved attr idents `db/query` (`db::query()`),
     `db/ident` (`db::ident()`), `db/cardinality` (`db::cardinality()`) — `fn`s, not consts, since `Attr` owns a
     `String`.
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
      `TrieStorageWrite`/`Trie(TrieWriteError)`/`HighBitInValue`/`NoSpaceInValueTable`).
      Re-exported at the crate root.
   - `src/find/` — the `Find` trait (`select()` + `where_() -> Vec<Atom>` + `process(FindResult)`; default `apply`
     runs `db_trie::find`; `FindResult` = `Vec<HashMap<String, Val>>`, lives at `find/types/find_result.rs`).
     Impls: `all_eins`,
     `attrs_of_ein`, `binds_for_attr`, `eins_with_attr`, `vals_in_slot`; each compiles to a datalog `rule` headed by
     `db/query`.
   - `src/query.rs` — `DbQuery` trait (`find`, `get`, `find_val`, `get_val`), implemented by `Db` and `DbReader`.
    - `src/reader.rs` — `DbReader<S: ReadStorage>`: `DbReader::load(db)` (sync, infallible) *borrows* a `Db<T>` and
      builds an
     independent snapshot via `T::Snapshot` (bound `T: ReadWriteStorage<Snapshot = S>`, so `T` stays inferable
     from the `Db`); the `Db` remains usable, and writes after `load` are invisible to the reader.
   - `src/transact.rs` — `Db::transact(self, datoms)` consumes and returns a new `Db`: resolves temps via `EntEid`,
     inserts values in the val table, `with_update` per datom, `set_max_tx(tx + 1)`, writes `MaxEid`, commits.
   - `src/handle.rs` — `DbHandle<S: ReadWriteStorage + Send + Sync + 'static>` owns the `Db` in a worker task;
     `transact` sends datoms over an mpsc channel, `to_reader()` returns a `DbReader<S::Snapshot>` snapshot; errors
     as `HandleError` (`TaskClosed`/`TaskFailed`/`TransactFailed`/`LoadFailed`).
   - `src/pull/` — `Pull` trait (`Serialize + Deserialize`; `attrs()` / `into_datoms()` / `pull(&Db, eid)`),
     `errors::RegisterError::DuplicateAttr`, tests in `pull/tests/`.
  - The read-only query machinery (`find`, datalog, `ev_stream`, `val_table::query`) is generic over
    `T: TrieQuery<HandleTrieConfig>`, so it works for `Trie` and `TrieReader` alike; it never needs
     `storage()`. Helpers that yield sub-tries return `T::Subtrie`. Unit tests are `#[cfg(test)]` beside the code (e.g. in
     `query.rs`, `find/mod.rs`, `crate_services/datalog/mod.rs`, `crate_services/val_table.rs`); integration tests in
      `crates/sky-db/tests/` (cardinality, db_reader, file_db, handle, mem_db, multiple_entities).
4. `crates/sky-server` — socket server over sky-db. `shared/` — the wire protocol (`SocketRequest`/`SocketResponse`,
   serde-only, no tokio/sky-db); `server/` — `process_socket_requests` (turns `SocketRequest`s from a `Stream` into
   `SocketResponse`s on a `Sink`; a closed request stream ends the connection task) + `server/storage`
   (`StorageService`: `subscribe()`,
   `read_storage_head()`, `read_slot_base()`, `transact()`; runs `Db` in a worker task, one-shot answers per
   request, and a `broadcast` channel that carries only `StorageBroadcastEvent::NewHead` — failures are logged by
   the worker and never broadcast; a failed transact kills the worker (db lost) and the unanswered one-shot maps
   to `StorageServiceError::TransactFailed`). The `server` module is behind the `server` cargo feature (default
   on); disabling it leaves only `shared` for client-side consumers. The optional `axum-ws` feature (implies
   `server`) adds `server/axum_ws` — websocket transport glue (`serve_websocket(socket, storage)`: JSON text
   frames of `SocketRequest`/`SocketResponse` routed through `process_socket_requests`); hosts own the axum route
   registration.

## Skybase frontend layout

Within `crates/skybase/src`:

- `routes/` — route-level/page components, one module per route (`routes/home.rs`). Pages own route wiring and data
  fetching (via `#[server]` calls in `api/` or resources); they are mounted in `<Route>`s in `app.rs`.
- `components/` — reusable presentational components with no data logic; they receive everything as props. If a
  component fetches or depends on `skybase::db`/`sky-db` data directly, it belongs in `routes/` (or its data should be loaded
  in a `route` and passed down). Exception (temporary): `components/wss_sandbox.rs` — the websocket playground, driven
  by `leptos-use`'s `use_websocket::<String, String, FromToStringCodec>("/ws")` over the axum `/ws` route, using the
  typed `sky_server::shared::{SocketRequest, SocketResponse}` protocol.
- `api/` — all `#[server]` functions (isomorphic: the same definition compiles to a client stub under `hydrate` and a
  server impl under `ssr`). Keep them out of components.
- `state/` — app-wide shared context: signals/resources and the types provided with `provide_context` / read with
  `expect_context`.
- `app.rs` — the `App` root (Router + `shell`); `server/mod.rs` — the axum `serve()` glue (ssr-only): mounts the
  leptos routes + a `/ws` route (`sky-server`'s `axum_ws::serve_websocket` over a `StorageService` carried in an
  axum `Extension`). Note: the websocket `StorageService` runs its own db for now — the `DbHandle` db in
  `skybase::db` is temporary and will be removed.

## Key gotchas

- **Bit-width constraints are strict.** Trie keys are 31-bit (non-negative `i32`); negative keys panic with
  `assertion failed: value >= 0`. Trie values are `TrieValue::U32(u32)` (32-bit). `Ein` (entity id) and `SlotBaseId`
  are non-negative `i32`s. `TrieValue`s stored in the EAVT/AEVT indexes pack `Dir` into bit 28 (`0x1000_0000`) and
  a 28-bit `Txid` into the low bits, so tx ids are capped below 2^28.
- **`SlotBaseId::ZERO` is the empty base.** It is never stored; every storage returns an empty `SlotBase` for it and
  appends start at id 1. Reading any other id that the storage (or snapshot) never assigned is a programming error
  and panics with a descriptive assert — ids come from committed map bases; external id sources (e.g.
  sky-server's `read_slot_base`) must range-check against `max_id` before calling `read`.
- **`Attr` owns a `String`** (attribute idents), not an integer or a `&'static str`. Schema attributes must be
  declared up front: `Db::new(storage, db_spec)` takes `impl Into<DbSpec>` (`[&str; N]`/`Vec<&str>`, `[Attr; N]`,
  `Vec<Attr>`, or `[AttrSpec; N]` for
  cardinality), while `Db::load(storage)` rebuilds the whole schema from the trie. Build an `Attr` with
  `Attr::from("...")`.
  Entity ids 0–2 are reserved (`Ein::DB_IDENT`,
  `Ein::DB_CARDINALITY`, `Ein::DB_MAX`); fresh eins (attributes at `Db::new`, temps at `transact`) are handed out by
  `MaxEid`, which starts at `Ein::DB_MAX`.
- **`Db` is immutable-value / consumed-ownership.** `Db::transact(...)` consumes `self` and returns a new `Db`. Get the
  underlying storage back with `db.close()` before re-`load`ing.
- **Generic bounds are pervasive.** Any struct/fn mentioning `Trie<S>` or `Db<S>` needs `S: ReadWriteStorage` (plus
  `TrieReadPolicy<Config = HandleTrieConfig>` where sky-db queries a handle trie). Because `ReadStorage:
  TrieReadPolicy`, the config flows through `S::Config`.
  Read-only sub-tries are owned `TrieReader` snapshots (`trie.view()`, `to_subtrie_from_value()`, `subtrie_stream()`);
  every `ReadStorage` provides `Snapshot`/`snapshot()`, which makes snapshots cheap: mem readers Arc-share the
  base pool and only capture `max_id`/`root`, file readers copy a `PathBuf`/`max_id`/`root`. Outside sky-db, use the
  traits and storages directly from `sky_types::storage` (`ReadStorage`/`ReadWriteStorage`, `MemStorage`/`FileStorage`).
- **Query methods live on the config-generic `TrieQuery<C: TrieConfig>` trait** (in `sky_types::trie`; `root`,
  `query_value`, `query_keys_values`, `deep_query_value`, `u32_stream`, `subtrie_stream`,
  `to_subtrie_from_value`, plus
  `type Subtrie: TrieQuery<C>`) — all required,
  no default bodies. Its supertrait `StorageTrieQuery<S>` adds `storage()` only (implementors provide `root()` via
  `TrieQuery` and `storage()` via `StorageTrieQuery`). Calling a query method needs
  `TrieQuery` in scope (it comes with `use crate::trie::prelude::*`); mutation methods (`insert`,
  `deep_insert`, `commit`) stay inherent on `Trie`. `subtrie_stream`/`to_subtrie_from_value` yield
  `Self::Subtrie`. `TrieReader` connects to a `ReadStorage` only (e.g. `storage.snapshot()`).
- **Errors are layered.** sky-types owns the storage errors `ReadStorageError` (`Io`/`Decode`) and
  `WriteStorageError` (`Io`/`Encode`), plus the trie errors `TrieQueryError` (`SystemError`) and `TrieInsertError`
   (`ExpectedMapBaseAtKey`/`Query`); sky-db's `ConnectError` (`Query`/`Transact`/
   `TrieStorageWrite`) embed them, and `QueryError`/`TransactError`
  (the latter embedding `QueryError`/`ReadStorageError`/`WriteStorageError`/`TrieInsertError`/`NoSpaceInValueTable`)
  live in `sky_types::db`, so `?` chains across crates work through `From` impls (`use crate::trie::prelude::*`
  brings the trie error types in scope).
- **`Ent` is either `Id(Ein)` or `Temp(String)`.** Temp entities get auto-assigned `Ein`s at transact time (see
  `src/db/types/ent_eid.rs`). Reusing the same temp ident in a tx rewrites the same entity, whereas separate txns
  create separate entities.
- `universal_hash::hash` (crate `universal-hash`, used via the extern prelude) is the hashing primitive;
  everything keys off it.

## Conventions

- Heavily async (`tokio`); most APIs return `impl Future` via `async fn` with `Result`.
- Symbol-heavy internal types: `Val` (db user value, `U32`/`String`), `TrieValue` (`U32`/`SubTrie(MapBase)`),
  `SlotBase` (a trie node's `Vec<Slot>`), `Slot` (`KeyValue`/`MapBase`), `MapBase` (`SlotMap` + handle),
  `TrieConfig`/`HandleTrieConfig` (the link config; `HandleTrieConfig`'s handle is `SlotBaseId`),
  `HashKey`/`DeepKey`, plus the db layer's `Dat`/`Datom`/`Ent`/`Dir`, `Attr`/`AttrSpec`/`DbSpec`/`Schema`,
  `Ein`/`Txid`/`Vid`/`MaxEid`/`EntEid`/`AttrEin` — don't confuse the similar names despite the overlap.
- `Dat::Val` vs `Dat::Ent` and `dir` (`Dir::In`/`Dir::Out`, i.e. add/delete) drive query semantics; see
  `src/types/datom.rs` (`datom::add` / `datom::del`).

# AGENTS.md

Datomic-like database library written in Rust (edition 2024), built on persistent
Hash Array Mapped Tries (HAMT). A Cargo workspace: library crate `hamt2` (in `crates/hamt2`). No CI, no README.

## Commands

- `cargo test` — runs all tests across the workspace (unit `#[cfg(test)]` and `tests/` integration). No special filters or services required; everything uses in-memory or temp-file storage.
- Single test: `cargo test <name>` (standard). Tests are `#[tokio::test]` async.
- `cargo leptos build` (run from the workspace root) — builds the `skybase` Leptos web app. `crates/skybase` is the Leptos frontend/backend; `crates/skydb` is its database layer (a `DbViewer`/`Db` wrapper over `hamt2` for reading the skybase version). `cargo leptos` needs `cargo-leptos` installed; it compiles the `hydrate` feature (wasm) and `ssr` feature (native axum server) targets.

## Architecture (read top-down in this order)

Layered, each layer building on the one below:

1. `src/space/` — storage abstraction (`Space` trait). Implementations: `mem` (in-memory) and `file` (temp/on-disk). A space stores blocks of `SlotValue`s addressed by `TableAddr`.
2. `src/trie/` — HAMT over a `Space`. `SpaceTrie` is the persistent map. Mutations return a new `SpaceTrie`; nothing is durable until `.commit(&mut space).await?`.
3. `src/db/` — the Datomic layer over `SpaceTrie`: `Datom` (`ent`/`attr`/`dat`/`dir`), schema, and queries (`find`/`pull`).

## Skybase frontend layout

Within `crates/skybase/src`:

- `routes/` — route-level/page components, one module per route (`routes/home.rs`). Pages own route wiring and data fetching (via `#[server]` calls in `api/` or resources); they are mounted in `<Route>`s in `app.rs`.
- `components/` — reusable presentational components with no data logic; they receive everything as props. If a component fetches or depends on `skydb`/`hamt2` data directly, it belongs in `routes/` (or its data should be loaded in a `route` and passed down).
- `api/` — all `#[server]` functions (isomorphic: the same definition compiles to a client stub under `hydrate` and a server impl under `ssr`). Keep them out of components.
- `state/` — app-wide shared context: signals/resources and the types provided with `provide_context` / read with `expect_context`.
- `app.rs` — the `App` root (Router + `shell`); `server.rs` — the axum `serve()` glue (ssr-only).

## Key gotchas

- **Bit-width constraints are strict.** Trie keys are 31-bit (non-negative `i32`); negative keys panic with `assertion failed: value >= 0`. Trie values are `u32` (32-bit). `Ein` (entity id) is a non-negative `i32`. Don't break these when touching `SpaceSlot` bit packing — commit `7c28590` swapped key/value widths deliberately.
- **`Attr` is `&'static str`** (attribute idents), not an integer. Schema attributes must be declared up front: `Db::new(space, [attrs])` / `Db::load(space, [attrs])` enumerate every `Attr` used. Loading with an undeclared attr fails with `LoadError::UnknownAttr`.
- **`Db` is immutable-value / consumed-ownership.** `Db::transact(...)` consumes `self` and returns a new `Db`. Get the underlying space back with `db.close()` before re-`load`ing.
- **`Ent` is either `Id(Ein)` or `Temp(&'static str)`.** Temp entities get auto-assigned `Ein`s at transact time (see `src/db/component/ent_eid.rs`). Reusing the same temp ident in a tx rewrites the same entity, whereas separate txns create separate entities.
- `hash::universal` is the hashing primitive; everything keys off it.

## Conventions

- Heavily async (`tokio`); most APIs return `impl Future` via `async fn` with `Result`.
- Symbol-heavy internal types: `Val` (user value, `U32`/`String`), `space::core::value::Value` (`U32`/`MapBase`), `MemValue`, `SlotValue` — don't confuse them despite similar names.
- `Dat::Val` vs `Dat::Ent` and `dir` (`Dir::In`/`Dir::Out`, i.e. add/delete) drive query semantics; see `src/db/core/datom/mod.rs` (`datom::add` / `datom::del`).

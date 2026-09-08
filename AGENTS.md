# AGENTS.md

Datomic-like database library written in Rust (edition 2024), built on persistent
Hash Array Mapped Tries (HAMT). Pure library crate `hamt2`; no binaries, no CI, no README.

## Commands

- `cargo test` — runs all tests (unit `#[cfg(test)]` and `tests/` integration). No special filters or services required; everything uses in-memory or temp-file storage.
- Single test: `cargo test <name>` (standard). Tests are `#[tokio::test]` async.

## Architecture (read top-down in this order)

Layered, each layer building on the one below:

1. `src/space/` — storage abstraction (`Space` trait). Implementations: `mem` (in-memory), `file` (temp/on-disk), and `doc` (iroh-based sync). A space stores blocks of `SlotValue`s addressed by `TableAddr`.
2. `src/trie/` — HAMT over a `Space`. `SpaceTrie` is the persistent map. Mutations return a new `SpaceTrie`; nothing is durable until `.commit(&mut space).await?`.
3. `src/db/` — the Datomic layer over `SpaceTrie`: `Datom` (`ent`/`attr`/`dat`/`dir`), schema, and queries (`find`/`pull`).

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

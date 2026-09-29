use crate::trie::{MapBase, TrieKey};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct Leg(MapBase, TrieKey);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct CursorPos {
    route: Vec<Leg>,
    active_root: MapBase,
}

impl Default for CursorPos {
    fn default() -> Self {
        Self {
            route: vec![],
            active_root: MapBase::empty(),
        }
    }
}

impl CursorPos {
    pub fn new(map_base: impl Into<MapBase>) -> Self {
        Self {
            route: vec![],
            active_root: map_base.into(),
        }
    }
    pub fn active_root(&self) -> &MapBase {
        &self.active_root
    }
    pub fn depth(&self) -> usize {
        self.route.len()
    }
    pub fn descend(&mut self, key: impl Into<TrieKey>, map_base: impl Into<MapBase>) {
        let leg = Leg(self.active_root, key.into());
        self.route.push(leg);
        self.active_root = map_base.into();
    }
    pub fn ascend(&mut self) -> Option<(TrieKey, MapBase)> {
        if let Some(Leg(map_base, key)) = self.route.pop() {
            let lower_map_base = self.active_root;
            self.active_root = map_base;
            Some((key, lower_map_base))
        } else {
            None
        }
    }
}

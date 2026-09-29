use crate::trie::{MapBase, TrieKey};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Leg {
    pub higher_root: MapBase,
    pub key: TrieKey,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct CursorPos {
    pub route: Vec<Leg>,
    pub active_root: MapBase,
}

impl CursorPos {
    pub fn new(map_base: MapBase) -> Self {
        Self {
            route: vec![],
            active_root: map_base,
        }
    }
    pub fn top_root(&self) -> MapBase {
        if let Some(leg) = self.route.first() {
            leg.higher_root
        } else {
            self.active_root
        }
    }
    pub fn active_root(&self) -> MapBase {
        self.active_root
    }
    pub fn depth(&self) -> usize {
        self.route.len()
    }
    pub fn descend(&mut self, key: impl Into<TrieKey>, map_base: impl Into<MapBase>) {
        let leg = Leg {
            higher_root: self.active_root,
            key: key.into(),
        };
        self.route.push(leg);
        self.active_root = map_base.into();
    }
    pub fn ascend(&mut self) -> Option<(TrieKey, MapBase)> {
        if let Some(Leg {
            higher_root: map_base,
            key,
        }) = self.route.pop()
        {
            let lower_map_base = self.active_root;
            self.active_root = map_base;
            Some((key, lower_map_base))
        } else {
            None
        }
    }
}

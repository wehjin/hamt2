#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Vid(i32);

impl Vid {
    pub fn from_id(i: i32) -> Self {
        Self(i)
    }
    pub fn to_id(&self) -> i32 {
        self.0
    }
}

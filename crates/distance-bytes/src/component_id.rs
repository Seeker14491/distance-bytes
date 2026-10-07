use std::fmt;

/// A value of the game's `ComponentID` enum. Values the game doesn't define are allowed so that
/// any file can be read.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ComponentId(pub i32);

impl ComponentId {
    /// The name of this ID in the game's enum.
    pub fn name(self) -> Option<&'static str> {
        crate::generated::component_ids::NAMES
            .binary_search_by_key(&self.0, |&(v, _)| v)
            .ok()
            .map(|i| crate::generated::component_ids::NAMES[i].1)
    }

    pub fn from_name(name: &str) -> Option<Self> {
        crate::generated::component_ids::NAMES
            .iter()
            .find(|&&(_, n)| n == name)
            .map(|&(v, _)| ComponentId(v))
    }
}

impl fmt::Debug for ComponentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.name() {
            Some(name) => write!(f, "{name}({})", self.0),
            None => write!(f, "ComponentId({})", self.0),
        }
    }
}

impl fmt::Display for ComponentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.name() {
            Some(name) => f.write_str(name),
            None => write!(f, "{}", self.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names() {
        assert_eq!(ComponentId::TRANSFORM.name(), Some("Transform"));
        assert_eq!(
            ComponentId::from_name("TeleporterEntrance").map(|id| id.name()),
            Some(Some("TeleporterEntrance"))
        );
        assert_eq!(ComponentId(100_000).name(), None);
    }
}

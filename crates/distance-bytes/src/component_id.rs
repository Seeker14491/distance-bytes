use std::fmt;
use std::sync::OnceLock;

use crate::generated::classes::CLASSES;
use crate::generated::component_ids::NAMES;
use crate::io::mark;

/// A value of the game's `ComponentID` enum. Values the game doesn't define are allowed so that
/// any file can be read.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ComponentId(pub i32);

impl ComponentId {
    /// The name of this ID in the game's enum.
    pub fn name(self) -> Option<&'static str> {
        NAMES
            .binary_search_by_key(&self.0, |&(v, _)| v)
            .ok()
            .map(|i| NAMES[i].1)
    }

    pub fn from_name(name: &str) -> Option<Self> {
        static BY_NAME: OnceLock<Vec<(&str, i32)>> = OnceLock::new();
        let by_name = BY_NAME.get_or_init(|| {
            let mut v: Vec<_> = NAMES.iter().map(|&(v, n)| (n, v)).collect();
            v.sort_unstable();
            v
        });
        by_name
            .binary_search_by_key(&name, |&(n, _)| n)
            .ok()
            .map(|i| ComponentId(by_name[i].1))
    }

    /// The version the game writes for this component type, if the game has a class for it.
    pub fn written_version(self) -> Option<i32> {
        self.class().map(|(version, _)| version)
    }

    /// The scope mark the game writes for this component type: [`mark::SERIALIZABLE`] for
    /// wrappers of built-in Unity components, otherwise [`mark::SERIAL_COMPONENT`].
    pub fn written_mark(self) -> i32 {
        match self.class() {
            Some((_, true)) => mark::SERIALIZABLE,
            _ => mark::SERIAL_COMPONENT,
        }
    }

    /// `(version written, built-in component wrapper)`.
    fn class(self) -> Option<(i32, bool)> {
        CLASSES
            .binary_search_by_key(&self.0, |&(id, _, _)| id)
            .ok()
            .map(|i| (CLASSES[i].1, CLASSES[i].2))
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
    use crate::components::ComponentData;

    #[test]
    fn names() {
        assert_eq!(ComponentId::TRANSFORM.name(), Some("Transform"));
        assert_eq!(
            ComponentId::from_name("TeleporterEntrance").map(|id| id.name()),
            Some(Some("TeleporterEntrance"))
        );
        assert_eq!(ComponentId::from_name("NoSuchComponent"), None);
        assert_eq!(ComponentId(100_000).name(), None);
        for &(value, name) in NAMES {
            assert_eq!(ComponentId::from_name(name), Some(ComponentId(value)));
        }
    }

    #[test]
    fn classes() {
        assert_eq!(ComponentId::TRANSFORM.written_mark(), mark::SERIALIZABLE);
        assert_eq!(ComponentId::GROUP.written_mark(), mark::SERIAL_COMPONENT);
        assert_eq!(ComponentId::TELEPORTER_ENTRANCE.written_version(), Some(4));
    }

    /// The latest version each typed body decodes is the version the game writes.
    #[test]
    fn decoded_versions_end_at_the_written_version() {
        for &(id, version, _) in CLASSES {
            let id = ComponentId(id);
            if let Some(data) = (0..=version).find_map(|v| ComponentData::sample(id, v)) {
                assert_eq!(data.latest_version(), version, "{id}");
            }
        }
    }
}

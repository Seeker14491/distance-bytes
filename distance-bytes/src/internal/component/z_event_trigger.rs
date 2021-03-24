use crate::internal::{util, Serializable, Visitor};
use anyhow::Error;
use serde::{Deserialize, Serialize};
use std::mem;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct ZEventTrigger {
    pub event_name: String,
    pub one_shot: bool,
}

impl Default for ZEventTrigger {
    fn default() -> Self {
        ZEventTrigger {
            event_name: "Event 0".to_owned(),
            one_shot: false,
        }
    }
}

impl Serializable for ZEventTrigger {
    const VERSION: i32 = 2;

    fn accept<V: Visitor>(&mut self, mut visitor: V, version: i32) -> Result<(), Error> {
        if version <= 1 {
            let mut hash = util::hash_to_i32(&self.event_name);
            visitor.visit_i32("Event ID", &mut hash)?;
            self.event_name = format!("Event {}", hash);
        } else if version == 2 {
            let mut event_name = Some(mem::take(&mut self.event_name));
            visitor.visit_string("eventName_", &mut event_name)?;
            self.event_name = event_name.unwrap();
        }

        if version >= 1 {
            visitor.visit_bool("oneShot_", &mut self.one_shot)?;
        } else {
            self.one_shot = true;
        }

        Ok(())
    }
}

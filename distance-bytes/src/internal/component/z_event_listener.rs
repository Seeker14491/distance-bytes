use crate::internal::{util, Serializable, Visitor};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::mem;

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ZEventListener {
    pub event_name: String,
    pub delay: f32,
}

impl Default for ZEventListener {
    fn default() -> Self {
        ZEventListener {
            event_name: "Event 0".to_owned(),
            delay: 0.0,
        }
    }
}

impl Serializable for ZEventListener {
    const VERSION: i32 = 2;

    fn accept<V: Visitor>(&mut self, mut visitor: V, version: i32) -> Result<()> {
        if version == 0 {
            let mut hash = util::hash_to_i32(&self.event_name);
            visitor.visit_i32("Event ID", &mut hash)?;
            self.event_name = format!("Event {}", hash);
        } else if version >= 1 {
            let mut event_name = Some(mem::take(&mut self.event_name));
            visitor.visit_string("eventName_", &mut event_name)?;
            self.event_name = event_name.unwrap();

            if version >= 2 {
                visitor.visit_f32("delay_", &mut self.delay)?;
            }
        }

        Ok(())
    }
}

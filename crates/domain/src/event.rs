use serde::Serialize;

use crate::{Id, Note};

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Event {
    Created { note: Note },
    Updated { note: Note },
    Deleted { id: Id },
}

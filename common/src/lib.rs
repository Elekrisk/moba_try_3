use serde::{Deserialize, Serialize};

pub mod network;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Side {
    Red,
    Blue,
}

impl Side {
    pub const ALL: [Side; 2] = [Side::Red, Side::Blue];
}

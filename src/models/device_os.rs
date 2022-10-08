use serde::{Serialize, Deserialize};


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DeviceOs {
    Android = 1,
    Windows = 2,
    Linux = 3,
}
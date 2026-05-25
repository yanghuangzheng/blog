use tokio::sync::OnceCell;

use crate::other::structs;

pub static ALL_CONFIG: OnceCell<structs::AllConfig> = OnceCell::const_new();

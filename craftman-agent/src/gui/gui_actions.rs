use crate::mods::configs::Configs;
use tokio::sync::oneshot;

pub enum ConfigRequest {
    Request(oneshot::Sender<Configs>),
    Edit(oneshot::Sender<EditRequestReturn>, Configs),
}

pub enum EditRequestReturn {
    Edited,
    EditInvalid(String),
}

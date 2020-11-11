pub mod rotel;

use async_trait::async_trait;
use color_eyre::eyre::Result;

#[async_trait]
pub trait EntityController {
    async fn run(&self) -> Result<()>;
}

#[derive(Debug)]
pub enum EntityMessage {
    Volume(u8),
    Mute(bool),
    Source(EntitySource),
}

#[derive(Debug)]
pub struct EntitySource {
    pub name: String,
}

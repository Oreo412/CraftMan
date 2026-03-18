use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum property {
    AllowFlight,
    Difficulty,
    Gamemode,
    Hardcore,
    Whitelist,
    PVP,
    GenerateStructures,
    MOTD(String),
    MaxPlayers(u32),
    AllowNether,
    MaxWorldSize(u32),
    ViewDistance(u32),
    SimulationDistance(u32),
    SpawnProtection(u32),
    SpawnNPC,
    SpawnAnimals,
    SpawnMonsters,
}

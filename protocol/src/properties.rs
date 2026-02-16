use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum property {
    allow_flight,
    difficulty,
    gamemode,
    hardcore,
    whitelist,
    pvp,
    generate_structures,
    motd(String),
    max_players(u32),
    allow_nether,
    max_world_size(u32),
    view_distance(u32),
    simulation_distance(u32),
    spawn_protection(u32),
    spawn_npcs,
    spawn_animals,
    spawn_monsters,
}

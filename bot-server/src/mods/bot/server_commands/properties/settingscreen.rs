use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingScreen {
    WorldGeneration,
    Gameplay,
    Admin,
}

impl fmt::Display for SettingScreen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SettingScreen::WorldGeneration => "world",
            SettingScreen::Gameplay => "gameplay",
            SettingScreen::Admin => "admin",
        };
        write!(f, "{s}")
    }
}

impl FromStr for SettingScreen {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "world" => Ok(SettingScreen::WorldGeneration),
            "gameplay" => Ok(SettingScreen::Gameplay),
            "admin" => Ok(SettingScreen::Admin),
            _ => Err(()),
        }
    }
}

use super::paths::open_data;
use serde::de::Deserializer;
use serde::Deserialize;
use std::collections::HashMap;

// TODO deserialize into a temporary struct, convert it into a nicer version of this
// TODO type providers?
#[derive(Deserialize)]
pub struct Mod {
    name: String,

    pub adds_tags: Vec<String>,
    pub domain: Domain,
    pub generation_type: GenType,

    // each group is present at most once on an item
    pub group: String,
    pub generation_weights: Vec<SpawnWeight>,
    pub spawn_weights: Vec<SpawnWeight>,

    #[serde(deserialize_with = "deserialize_optional_buff")]
    grants_buff: Option<Buff>,
    grants_effects: Vec<GrantedEffect>,
    stats: Vec<Stat>,
    pub required_level: u8,

    // index into additional data
    r#type: String,
}

pub fn load_mods() -> HashMap<String, Mod> {
    serde_json::from_reader(open_data("mods.min.json")).unwrap()
}

impl Mod {
    pub fn generate(&self) -> ModInstance {
        ModInstance {
            rolls: self.stats.iter().map(|s| s.roll()).collect(),
            prototype: self,
        }
    }
}

pub struct ModInstance<'a> {
    pub rolls: Vec<i32>,
    pub prototype: &'a Mod,
}

impl<'a> ModInstance<'a> {
    pub fn stats(&'a self) -> impl Iterator<Item = StatRoll<'a>> {
        self.prototype
            .stats
            .iter()
            .zip(self.rolls.iter())
            .map(|(stat, &roll)| StatRoll { id: &stat.id, roll })
    }
}

pub struct StatRoll<'a> {
    pub id: &'a str,
    pub roll: i32,
}

#[derive(Deserialize, PartialEq, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Domain {
    Item,
    AbyssJewel,
    Area,
    Misc,
    Flask,
    Crafted,
    Delve,
    Atlas,
    /// This one appear at least on all currency bases
    Undefined,
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GenType {
    Prefix,
    Suffix,
    Unique,
    Corrupted,
    Enchantment,
    BlightTower,
    Tempest,
}

#[derive(Deserialize)]
pub struct SpawnWeight {
    pub tag: String,
    pub weight: u32,
}

#[derive(Deserialize)]
pub struct Buff {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub range: u32,
}

fn deserialize_optional_buff<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Buff>, D::Error> {
    Buff::deserialize(deserializer).map(|b| if b.id == "" { None } else { Some(b) })
}

#[derive(Deserialize)]
pub struct GrantedEffect {
    pub granted_effect_id: String,
    pub level: u8,
}

#[derive(Deserialize)]
pub struct Stat {
    pub id: String,
    pub min: i32,
    pub max: i32,
}

impl Stat {
    fn roll(&self) -> i32 {
        use rand::Rng;
        rand::thread_rng().gen_range(self.min, self.max + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::super::mods::load_mods;

    #[test]
    fn test_deserialize() {
        load_mods();
    }
}

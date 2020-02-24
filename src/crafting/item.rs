use super::mod_describer::ModDescriber;
use super::mods::*;
use super::paths::open_data;
use rand::distributions::{weighted::WeightedIndex, Distribution};
use rand::thread_rng;
use std::collections::HashMap;
use std::fmt;
use GenType::*;

pub struct Item<'a> {
    pub base: &'a Base<'a>,
    pub explicits: Vec<ModInstance<'a>>,

    pub name: String,
    pub item_level: u8,
    pub rarity: Rarity,
    pub quality: Quality,
}

impl<'a> Item<'a> {
    pub fn properties(&self) -> Properties {
        let mut props = self.base.properties.clone();

        let mut armour_percent = 100;
        let mut evasion_percent = 100;
        let mut energy_shield_percent = 100;
        let mut pdamage_percent = 100;
        let mut crit_percent = 100;
        let mut attack_speed_percent = 100;

        // TODO implicits
        for s in self.explicits.iter().flat_map(|e| e.stats()) {
            match s.id {
                "local_base_physical_damage_reduction_rating" => props.armour += s.roll,
                "local_base_evasion_rating" => props.evasion += s.roll,
                "local_energy_shield" => props.energy_shield += s.roll,
                "local_minimum_added_physical_damage" => props.physical_damage_min += s.roll,
                "local_maximum_added_physical_damage" => props.physical_damage_max += s.roll,
                "local_item_quality_+" => props.quality += s.roll as u8,
                "local_additional_block_chance_%" => props.block += s.roll as u8,

                "local_physical_damage_reduction_rating_+%" => armour_percent += s.roll,
                "local_evasion_rating_+%" => evasion_percent += s.roll,
                "local_energy_shield_+%" => energy_shield_percent += s.roll,
                "local_armour_and_evasion_+%" => {
                    armour_percent += s.roll;
                    evasion_percent += s.roll;
                }
                "local_armour_and_energy_shield_+%" => {
                    armour_percent += s.roll;
                    energy_shield_percent += s.roll;
                }
                "local_evasion_and_energy_shield_+%" => {
                    evasion_percent += s.roll;
                    energy_shield_percent += s.roll;
                }
                "local_armour_and_evasion_and_energy_shield_+%" => {
                    armour_percent += s.roll;
                    evasion_percent += s.roll;
                    energy_shield_percent += s.roll;
                }
                "local_physical_damage_+%" => pdamage_percent += s.roll,
                "local_critical_strike_chance_+%" => crit_percent += s.roll,
                "local_attack_speed_+%" => attack_speed_percent += s.roll,

                _ => {}
            }
        }

        props.quality += self.quality.amount;

        if self.quality.kind == QualityKind::Normal {
            let amount = props.quality as i32;
            armour_percent += amount;
            evasion_percent += amount;
            energy_shield_percent += amount;
            pdamage_percent += amount;
        }

        props.armour = props.armour * armour_percent / 100;
        props.evasion = props.evasion * evasion_percent / 100;
        props.energy_shield = props.energy_shield * energy_shield_percent / 100;
        props.physical_damage_min = props.physical_damage_min * pdamage_percent / 100;
        props.physical_damage_max = props.physical_damage_max * pdamage_percent / 100;
        props.critical_strike_chance = props.critical_strike_chance * crit_percent / 100;
        props.attack_time = props.attack_time * 100 / attack_speed_percent;

        props
    }

    pub fn apply_alchemy(&mut self, all_mods: impl Iterator<Item = &'a Mod> + Clone) {
        if self.rarity != Rarity::Normal {
            return;
        }

        let explicit_count = Item::roll_mod_count(Rarity::Rare, &self.base.item_class);

        for _ in 0..explicit_count {
            self.add_mod(all_mods.clone(), Rarity::Rare);
        }

        self.rarity = Rarity::Rare;
    }

    pub fn roll_mod_count(rarity: Rarity, item_class: &str) -> usize {
        if rarity == Rarity::Magic {
            if rand::random::<bool>() {
                1
            } else {
                2
            }
        } else if rarity == Rarity::Rare {
            if item_class == "Jewel" || item_class == "AbyssJewel" {
                WeightedIndex::new(vec![13, 7])
                    .unwrap()
                    .sample(&mut thread_rng())
                    + 3
            } else {
                WeightedIndex::new(vec![8, 3, 1])
                    .unwrap()
                    .sample(&mut thread_rng())
                    + 4
            }
        } else {
            0
        }
    }

    fn add_mod(&mut self, all_mods: impl Iterator<Item = &'a Mod> + Clone, target_rarity: Rarity) {
        let scale = self.make_scale(target_rarity);

        let mut weighted = all_mods.filter_map(|m| {
            let w = scale.mod_weight(m);
            if w == 0 {
                None
            } else {
                Some((m, w))
            }
        });

        let i = WeightedIndex::new(weighted.clone().map(|(_, w)| w))
            .unwrap()
            .sample(&mut thread_rng());
        let new = weighted.nth(i).unwrap().0.generate();
        self.explicits.push(new);
    }

    fn make_scale(&self, target_rarity: Rarity) -> Scale {
        let affixlimit = if target_rarity == Rarity::Magic {
            1
        } else if target_rarity == Rarity::Rare {
            if self.base.item_class == "Jewel" || self.base.item_class == "AbyssJewel" {
                2
            } else {
                3
            }
        } else {
            0
        };

        Scale {
            open_prefix: self.count_generation_type(Prefix) < affixlimit,
            open_suffix: self.count_generation_type(Suffix) < affixlimit,
            domain: self.base.domain,
            item_level: self.item_level,
            groups: self
                .explicits
                .iter()
                .map(|e| &e.prototype.group[..])
                .collect(),

            // TODO influences
            tags: self
                .explicits
                .iter()
                .flat_map(|e| &e.prototype.adds_tags)
                .chain(self.base.tags.iter())
                .map(|x| &x[..])
                .collect(),
        }
    }

    fn count_generation_type(&self, t: GenType) -> usize {
        self.explicits
            .iter()
            .filter(|e| e.prototype.generation_type == t)
            .count()
    }

    pub fn display(&self, md: &ModDescriber, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Rarity: {:?}", self.rarity)?;
        writeln!(f, "{}", self.name)?;

        writeln!(f, "--------")?;

        writeln!(f, "{}", self.base.item_class)?;

        let props = self.properties();

        // TODO quality

        if props.block != 0 {
            writeln!(f, "Chance to Block: {}%", props.block)?;
        }
        if props.armour != 0 {
            writeln!(f, "Armour: {}", props.armour)?;
        }
        if props.evasion != 0 {
            writeln!(f, "Evasion: {}", props.evasion)?;
        }
        if props.energy_shield != 0 {
            writeln!(f, "Energy Shield: {}", props.energy_shield)?;
        }
        if props.physical_damage_max != 0 {
            writeln!(
                f,
                "Physical Damage: {}-{}",
                props.physical_damage_min, props.physical_damage_max
            )?;
        }
        if props.critical_strike_chance != 0 {
            writeln!(
                f,
                "Critical Strike Chance: {:.2}",
                props.critical_strike_chance as f32 / 100.0
            )?;
        }
        if props.attack_time != 0 {
            writeln!(
                f,
                "Attacks per Second: {:.2}",
                1000.0 / props.attack_time as f32
            )?;
        }

        writeln!(f, "--------")?;

        writeln!(f, "Item Level: {}", self.item_level)?;

        // TODO implicits

        if self.explicits.len() > 0 {
            writeln!(f, "--------")?;
            for e in &self.explicits {
                md.describe(e, f)?;
            }
        }

        Ok(())
    }
}

struct Scale<'a> {
    open_prefix: bool,
    open_suffix: bool,
    item_level: u8,
    domain: Domain,
    groups: Vec<&'a str>,
    tags: Vec<&'a str>,
}
impl Scale<'_> {
    fn mod_weight(&self, m: &Mod) -> u32 {
        if m.domain != self.domain
            || !(m.generation_type == Prefix && self.open_prefix
                || m.generation_type == Suffix && self.open_suffix)
            || m.required_level > self.item_level
            || self.groups.contains(&&m.group[..])
        {
            return 0;
        }

        // TODO catalyst stuff?
        m.spawn_weights
            .iter()
            .find(|w| self.tags.contains(&&w.tag[..]))
            .map(|w| w.weight)
            .unwrap_or(0)
            * m.generation_weights
                .iter()
                .find(|w| self.tags.contains(&&w.tag[..]))
                .map(|w| w.weight)
                .unwrap_or(100)
            / 100
    }
}
#[derive(Debug, PartialEq)]
pub enum Rarity {
    Normal,
    Magic,
    Rare,
    Unique,
}

pub struct Quality {
    pub amount: u8,
    pub kind: QualityKind,
}

#[derive(PartialEq)]
pub enum QualityKind {
    Normal,
    Imbued,
}

use serde::Deserialize;

#[derive(Deserialize)]
struct RawBase {
    domain: Domain,
    // TODO make enum
    item_class: String,
    tags: Vec<String>,
    name: String,

    properties: Properties,
    implicits: Vec<String>,
    requirements: Option<Requirements>,
    inventory_width: u8,
    inventory_height: u8,
}

pub struct Base<'a> {
    pub domain: Domain,
    // TODO make enum
    pub item_class: String,
    pub tags: Vec<String>,
    pub name: String,

    pub properties: Properties,
    pub implicits: Vec<&'a Mod>,
    pub requirements: Option<Requirements>,
    pub inventory_width: u8,
    pub inventory_height: u8,
}

pub fn load_bases(mods: &HashMap<String, Mod>) -> Vec<Base> {
    let mut m: std::collections::HashMap<String, RawBase> =
        serde_json::from_reader(open_data("base_items.min.json")).unwrap();
    m.drain()
        .map(|(_, v)| v)
        .map(|b| Base {
            domain: b.domain,
            item_class: b.item_class,
            tags: b.tags,
            name: b.name,
            properties: b.properties,
            implicits: b
                .implicits
                .iter()
                .map(|name| mods.get(name).unwrap())
                .collect(),
            requirements: b.requirements,
            inventory_width: b.inventory_width,
            inventory_height: b.inventory_height,
        })
        .collect()
}

#[derive(Deserialize)]
pub struct Requirements {
    pub strength: u32,
    pub dexterity: u32,
    pub intelligence: u32,
    pub level: u8,
}

#[derive(Deserialize, Clone)]
pub struct Properties {
    #[serde(default)]
    quality: u8,
    #[serde(default)]
    armour: i32,
    #[serde(default)]
    evasion: i32,
    #[serde(default)]
    energy_shield: i32,
    #[serde(default)]
    block: u8,
    #[serde(default)]
    attack_time: i32,
    #[serde(default)]
    critical_strike_chance: i32,
    #[serde(default)]
    physical_damage_min: i32,
    #[serde(default)]
    physical_damage_max: i32,
}

#[cfg(test)]
mod tests {
    use super::load_bases;
    #[test]
    fn test_deserialize() {
        load_bases(&super::super::mods::load_mods());
    }
}

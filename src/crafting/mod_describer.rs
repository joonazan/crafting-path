use super::mods::{ModInstance, StatRoll};
use super::paths::open_data;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;

pub struct ModDescriber<'a> {
    mod_to_description: HashMap<String, &'a StatDescription>,
}

impl<'a> ModDescriber<'a> {
    pub fn new(descriptions: &'a Vec<StatDescription>) -> Self {
        let mut mod_to_description = HashMap::new();

        for d in descriptions {
            for id in &d.ids {
                mod_to_description.insert(id.clone(), d);
            }
        }

        Self { mod_to_description }
    }
    pub fn describe(&self, m: &ModInstance, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut stats: Vec<StatRoll> = m.stats().collect();
        while let Some(s) = stats.last() {
            if let Some(d) = self.mod_to_description.get(s.id) {
                let mut rolls = vec![];
                for id in &d.ids {
                    let mut roll = 0;
                    for i in 0..stats.len() {
                        if stats[i].id == id {
                            roll = stats[i].roll;
                            stats.swap_remove(i);
                            break;
                        }
                    }
                    rolls.push(roll);
                }

                if let Some(s) = d
                    .alternatives
                    .iter()
                    .filter_map(|a| a.describe_if_match(&rolls))
                    .next()
                {
                    writeln!(f, "{}", s)?;
                } else {
                    writeln!(f, "ERROR: None of the description alternatives match.")?;
                }
            } else {
                println!("ERROR: Didn't find description for {}", s.id);
                stats.pop();
            }
        }

        Ok(())
    }
}

pub fn load_stat_descriptions() -> Vec<StatDescription> {
    serde_json::from_reader(open_data("stat_translations.min.json")).unwrap()
}

#[derive(Deserialize)]
pub struct StatDescription {
    #[serde(rename = "English")]
    alternatives: Vec<StatWording>,
    ids: Vec<String>,
}

#[derive(Deserialize)]
struct StatWording {
    condition: Vec<Range>,
    // TODO make enum
    format: Vec<String>,
    index_handlers: Vec<Vec<String>>,
    string: String,
}

impl StatWording {
    fn describe_if_match(&self, rolls: &Vec<i32>) -> Option<String> {
        if rolls.iter().zip(&self.condition).all(|(&r, range)| {
            range.min.map(|m| m <= r).unwrap_or(true) && range.max.map(|m| r <= m).unwrap_or(true)
        }) {
            Some(self.describe(rolls))
        } else {
            None
        }
    }

    fn describe(&self, rolls: &Vec<i32>) -> String {
        let mut res = self.string.clone();
        for i in 0..rolls.len() {
            if self.format[i] != "ignore" {
                res = res.replace(
                    &("{".to_string() + &i.to_string() + "}"),
                    &self.format[i].replace("#", &rolls[i].to_string()),
                );
                // TODO index_handlers
            }
        }

        res
    }
}

#[derive(Deserialize)]
struct Range {
    min: Option<i32>,
    max: Option<i32>,
}

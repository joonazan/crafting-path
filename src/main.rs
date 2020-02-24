mod crafting;
use crafting::item::*;
use crafting::mod_describer::*;
use crafting::mods::load_mods;
use std::fmt;

pub struct Fmt<F>(pub F)
where
    F: Fn(&mut fmt::Formatter) -> fmt::Result;

impl<F> fmt::Display for Fmt<F>
where
    F: Fn(&mut fmt::Formatter) -> fmt::Result,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (self.0)(f)
    }
}

fn main() {
    println!("Loading item database...");
    let mods = load_mods();
    let bases = load_bases(&mods);
    let descs = load_stat_descriptions();
    let md = ModDescriber::new(&descs);
    println!("Done!");

    for _ in 0..100 {
        let mut item = Item {
            explicits: vec![],
            name: "Imaginary Sword".to_string(),
            base: bases
                .iter()
                .filter(|b| b.item_class == "Two Hand Sword")
                .next()
                .unwrap(),
            item_level: 1,
            quality: Quality {
                amount: 20,
                kind: QualityKind::Normal,
            },
            rarity: Rarity::Normal,
        };

        item.apply_alchemy(mods.values());
        println!("{}", Fmt(|f| item.display(&md, f)));
    }
}

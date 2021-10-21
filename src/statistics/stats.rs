// Winrates code - from https://github.com/AnnikaCodes/randbats-winrates/blob/main/src/stats.rs
use indexmap::IndexMap;
use prettytable::*;

use crate::BattleToolsError;

#[derive(Copy, Clone)]
struct FinalStats {
    /// as percentage
    winrate: f32,
    deviations: f32,
}

/// Stores statistics about a pokemon
#[derive(Copy, Clone, Debug)]
struct PokemonStats {
    games: u32,
    wins: u32,
}

impl PokemonStats {
    /// Computes the number of standard deviations from the average
    fn final_stats(&self) -> FinalStats {
        let games = self.games as f32;
        let winrate = (self.wins as f32 / games) * 100.0;

        // Standard deviations formula courtesy of pyuk (@pyuk-bot on GitHub)
        let deviations = (winrate - 50.0) * games.sqrt() / 50.0;

        FinalStats {
            winrate,
            deviations,
        }
    }
}

#[derive(Debug)]
pub struct GameResult {
    species: String,
    won: bool,
}

/// Stores overall statistics
#[derive(Debug)]
pub struct Stats {
    /// Pokemon:statistics map
    pokemon: IndexMap<String, PokemonStats>,
    is_sorted: bool,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            pokemon: IndexMap::new(),
            is_sorted: false,
        }
    }

    pub fn sort(&mut self) {
        if !self.is_sorted {
            self.pokemon.sort_by(|_, a, _, b| {
                b.final_stats()
                    .deviations
                    .partial_cmp(&a.final_stats().deviations)
                    .unwrap()
            });
        }
    }

    pub fn process_json(min_elo: u64, json: &str) -> Result<Vec<GameResult>, BattleToolsError> {
        // ELO check
        for elo_property in ["p1rating.elo", "p2rating.elo"].iter() {
            if (gjson::get(json, elo_property).f32() as u64) < min_elo {
                // ignore
                return Ok(vec![]);
            }
        }

        let mut results = vec![];

        // (indices of parsed JSON)
        for (species_list_property, player_property) in
            [("p1team.#.species", "p1"), ("p2team.#.species", "p2")].iter()
        {
            // json[16] = the winner
            let won = gjson::get(json, player_property) == gjson::get(json, "winner");

            let species_list = gjson::get(json, species_list_property);
            for species in species_list.array() {
                results.push(GameResult {
                    species: Stats::normalize_species(species.str()),
                    won,
                });
            }
        }
        Ok(results)
    }

    pub fn add_game_results(&mut self, results: Vec<GameResult>) {
        if results.is_empty() {
            return;
        }

        self.is_sorted = false; // we're adding data so it isn't sorted anymore
        for result in results {
            let wins = if result.won { 1 } else { 0 };
            match self.pokemon.get_mut(&result.species) {
                Some(s) => {
                    s.wins += wins;
                    s.games += 1;
                }
                None => {
                    self.pokemon
                        .insert(result.species, PokemonStats { games: 1, wins });
                }
            };
        }
    }

    fn normalize_species(species: &str) -> String {
        if species.starts_with("Pikachu-") {
            String::from("Pikachu")
        } else if species.starts_with("Unown-") {
            String::from("Unown")
        } else if species == "Gastrodon-East" {
            String::from("Gastrodon")
        } else if species == "Magearna-Original" {
            String::from("Magearna")
        } else if species == "Genesect-Douse" {
            String::from("Genesect")
        } else if species.starts_with("Basculin-") {
            String::from("Basculin")
        } else if species.starts_with("Sawsbuck-") {
            String::from("Sawsbuck")
        } else if species.starts_with("Vivillon-") {
            String::from("Vivillon")
        } else if species.starts_with("Florges-") {
            String::from("Florges")
        } else if species.starts_with("Furfrou-") {
            String::from("Furfrou")
        } else if species.starts_with("Minior-") {
            String::from("Minior")
        } else if species.starts_with("Gourgeist-") {
            String::from("Gourgeist")
        } else if species.starts_with("Toxtricity-") {
            String::from("Toxtricity")
        } else {
            species.to_string()
        }
    }
}

impl super::StatsOutput for Stats {
    fn to_csv(&mut self) -> String {
        self.sort();

        self.pokemon
            .iter()
            .map(|(pokemon, stats)| {
                let fstats = stats.final_stats();
                [
                    pokemon.to_string(),
                    stats.games.to_string(),
                    stats.wins.to_string(),
                    fstats.winrate.to_string(),
                    fstats.deviations.to_string(),
                ]
                .join(",")
            })
            .intersperse(String::from("\n"))
            .collect()
    }

    fn to_human_readable(&mut self) -> String {
        let mut table = table!(["Rank", "Pokemon", "Deviations", "Winrate", "Games", "Wins"]);
        let mut cur_rank = 1;

        self.sort();

        for (pokemon, stats) in &self.pokemon {
            let fstats = stats.final_stats();

            let deviations = fstats.deviations.to_string();
            let mut winrate = fstats.winrate.to_string();
            winrate.push('%');

            table.add_row(row![
                cur_rank,
                pokemon,
                deviations,
                winrate,
                stats.games,
                stats.wins
            ]);
            cur_rank += 1;
        }

        table.to_string()
    }
}

#[cfg(test)]
mod unit_tests {
    extern crate test;
    use super::{super::StatsOutput, Stats};
    use lazy_static::lazy_static;
    use test::Bencher;

    lazy_static! {
        static ref SAMPLE_JSON: String = String::from(
            r#"{"winner":"Annika","seed":[1,1,1,1],"turns":2,"p1":"Annika","p2":"Rust Haters","p1team":[{"name":"Rotom","species":"Rotom-Fan","gender":"N","shiny":false,"gigantamax":false,"level":84,"moves":["airslash","voltswitch","willowisp","thunderbolt"],"ability":"Levitate","evs":{"hp":85,"atk":0,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":0,"def":31,"spa":31,"spd":31,"spe":31},"item":"Heavy-Duty Boots"},{"name":"Regirock","species":"Regirock","gender":"N","shiny":false,"gigantamax":false,"level":85,"moves":["curse","rockslide","rest","bodypress"],"ability":"Sturdy","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Chesto Berry"},{"name":"Conkeldurr","species":"Conkeldurr","gender":"","shiny":false,"gigantamax":false,"level":80,"moves":["facade","knockoff","machpunch","drainpunch"],"ability":"Guts","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Flame Orb"},{"name":"Reuniclus","species":"Reuniclus","gender":"","shiny":false,"gigantamax":false,"level":84,"moves":["trickroom","focusblast","psychic","shadowball"],"ability":"Magic Guard","evs":{"hp":85,"atk":0,"def":85,"spa":85,"spd":85,"spe":0},"ivs":{"hp":31,"atk":0,"def":31,"spa":31,"spd":31,"spe":0},"item":"Life Orb"},{"name":"Incineroar","species":"Incineroar","gender":"","shiny":false,"gigantamax":false,"level":80,"moves":["knockoff","uturn","earthquake","flareblitz"],"ability":"Intimidate","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Choice Scarf"},{"name":"Miltank","species":"Miltank","gender":"F","shiny":false,"gigantamax":false,"level":84,"moves":["healbell","bodyslam","earthquake","milkdrink"],"ability":"Sap Sipper","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Leftovers"}],"p2team":[{"name":"Drednaw","species":"Drednaw","gender":"","shiny":false,"gigantamax":false,"level":84,"moves":["stoneedge","swordsdance","superpower","liquidation"],"ability":"Swift Swim","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Life Orb"},{"name":"Pinsir","species":"Pinsir","gender":"","shiny":false,"gigantamax":false,"level":84,"moves":["closecombat","stoneedge","xscissor","knockoff"],"ability":"Moxie","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Choice Scarf"},{"name":"Pikachu","species":"Pikachu-Sinnoh","gender":"","shiny":false,"gigantamax":false,"level":92,"moves":["knockoff","volttackle","voltswitch","irontail"],"ability":"Lightning Rod","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Light Ball"},{"name":"Latios","species":"Latios","gender":"M","shiny":false,"gigantamax":false,"level":78,"moves":["dracometeor","calmmind","psyshock","roost"],"ability":"Levitate","evs":{"hp":85,"atk":0,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":0,"def":31,"spa":31,"spd":31,"spe":31},"item":"Soul Dew"},{"name":"Entei","species":"Entei","gender":"N","shiny":false,"gigantamax":false,"level":78,"moves":["flareblitz","stoneedge","extremespeed","sacredfire"],"ability":"Inner Focus","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Choice Band"},{"name":"Exeggutor","species":"Exeggutor-Alola","gender":"","shiny":false,"gigantamax":false,"level":86,"moves":["gigadrain","flamethrower","dracometeor","leafstorm"],"ability":"Frisk","evs":{"hp":85,"atk":0,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":0,"def":31,"spa":31,"spd":31,"spe":31},"item":"Choice Specs"}],"score":[0,2],"inputLog":[">lol you thought i'd leak someone's real input log"],"log":["|j|☆Annika","|j|☆Rust Hater","|player|p1|Annika|cynthia|1400","|player|p2|Rust Hater|cynthia|1100","|teamsize|p1|6","|teamsize|p2|6","|gametype|singles","|gen|8","|tier|[Gen 8] Random Battle","|rated|"],"p1rating":{"entryid":"75790599","userid":"annika","w":"4","l":4,"t":"0","gxe":46.8,"r":1516.9377700433,"rd":121.36211247153,"rptime":1632906000,"rpr":1474.7452159936,"rprd":115.09180605287,"elo":1400.4859871929,"col1":8,"oldelo":"1057.7590112468"},"p2rating":{"entryid":"75790599","userid":"rusthater","w":"4","l":5,"t":"0","gxe":41.8,"r":"1516.9377700433","rd":"121.36211247153","rptime":"1632906000","rpr":1434.9434039083,"rprd":109.84367373045,"elo":1130.7522733629,"col1":9,"oldelo":"1040.4859871929"},"endType":"normal","timestamp":"Wed Nov 1 1970 00:00:01 GMT-0400 (Eastern Daylight Time)","roomid":"battle-gen8randombattle-1","format":"gen8randombattle", "comment": "if you're curious - this is my own rating info & teams from my battles - no violation of privacy here!"}"#
        );
    }

    fn add_records(stats: &mut Stats, num: u32) {
        for _ in 0..num {
            let s = Stats::process_json(1050, &SAMPLE_JSON).unwrap();
            stats.add_game_results(s);
        }
    }

    #[bench]
    pub fn bench_process_json(b: &mut Bencher) {
        b.iter(|| Stats::process_json(1050, &SAMPLE_JSON));
    }

    #[bench]
    pub fn bench_process_and_add_json(b: &mut Bencher) {
        let mut stats = Stats::new();
        b.iter(|| {
            let s = Stats::process_json(1050, &SAMPLE_JSON).unwrap();
            stats.add_game_results(s);
        });
    }

    #[bench]
    pub fn bench_to_csv_10k(b: &mut Bencher) {
        let mut stats = Stats::new();
        add_records(&mut stats, 10000);
        b.iter(|| stats.to_csv());
    }

    #[bench]
    pub fn bench_to_prettytable_10k(b: &mut Bencher) {
        let mut stats = Stats::new();
        add_records(&mut stats, 10000);
        b.iter(|| stats.to_human_readable());
    }
}

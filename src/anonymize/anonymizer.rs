// From https://github.com/AnnikaCodes/anonbattle/blob/main/src/anonymizer.rs

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Mutex;

use lazy_static::*;
use regex::Regex;

use serde_json::{json, Value};

use crate::{id::to_id, BattleToolsError};

lazy_static! {
    static ref INPUTLOG_ANONYMIZER_REGEX: Regex = Regex::new(r#"name":".*","#).unwrap();
}

/// Tracks players
struct SharedState {
    players: HashMap<String, String>,
    pub current_battle_number: u32,
    cur_player_number: i32,
}

impl SharedState {
    fn new() -> Self {
        Self {
            players: HashMap::new(),
            current_battle_number: 0,
            cur_player_number: 0,
        }
    }

    fn anonymize_player(&mut self, userid: String) -> String {
        match self.players.get(&userid) {
            Some(anonymized) => anonymized.to_string(),
            None => {
                self.cur_player_number += 1;
                let num = self.cur_player_number;
                self.players.insert(userid, num.to_string());
                self.cur_player_number.to_string()
            }
        }
    }
}

/// Anonymizes string JSON while tracking state
pub struct Anonymizer {
    state: Mutex<SharedState>,
    /// Panics if player names sneak past
    is_safe: bool,
}

impl Anonymizer {
    pub fn new(is_safe: bool) -> Self {
        Self {
            state: Mutex::new(SharedState::new()),
            is_safe,
        }
    }

    /// Anonymizes a log.
    ///
    /// Returns a tuple: (json, battle_number)
    pub fn anonymize(&self, raw: &str) -> Result<(String, u32), BattleToolsError> {
        let json: serde_json::Map<String, Value> = serde_json::from_str(raw)?;

        let p1 = json["p1"]
            .as_str()
            .ok_or(format!("Bad JSON for p1 {}", json["p1"]))?;
        let p2 = json["p2"]
            .as_str()
            .ok_or(format!("Bad JSON for p2 {}", json["p2"]))?;
        let p1_id = to_id(p1);
        let p2_id = to_id(p2);

        let winner = json["winner"]
            .as_str()
            .ok_or(format!("Bad JSON for winner {}", json["winner"]))?
            .to_owned();

        let (p1_anon, p2_anon, winner_anon) = {
            let mut tracker = self.state.lock().unwrap();
            (
                tracker.anonymize_player(p1.to_string()),
                tracker.anonymize_player(p2.to_string()),
                tracker.anonymize_player(winner),
            )
        };

        let mut json_result = json.clone();
        // Anonymize
        json_result["p1"] = Value::String(p1_anon.clone());
        json_result["p2"] = Value::String(p2_anon.clone());
        json_result["winner"] = Value::String(winner_anon);

        json_result["p1rating"] = Value::Null;
        json_result["p2rating"] = Value::Null;
        json_result["roomid"] = Value::Null;

        // "Sat Nov 21 2020 17:05:04 GMT-0500 (Eastern Standard Time)" -> "Sat Nov 21 2020 17"
        let mut timestamp = json["timestamp"]
            .as_str()
            .ok_or(format!("Bad JSON for timestamp {}", json["timestamp"]))?
            .split(':')
            .collect::<Vec<&str>>()[0]
            .to_owned();
        timestamp.push_str(":XX");

        json_result["timestamp"] = json!(timestamp);

        let il = json["inputLog"]
            .as_array()
            .ok_or(format!("Bad JSON for inputlog {}", json["inputLog"]))?
            .iter();
        json_result["inputLog"] = serde_json::json!(il
            .filter_map(|inputlog_part| {
                let inputlog_part_string: &str = inputlog_part.as_str().unwrap();
                if inputlog_part_string.starts_with(">player p1") {
                    let res = INPUTLOG_ANONYMIZER_REGEX
                        .replace_all(inputlog_part_string, |_: &regex::Captures| {
                            format!("\"name\":\"{}\",", p1_anon)
                        });
                    return Some(json!(res));
                } else if inputlog_part_string.starts_with(">player p2") {
                    let res = INPUTLOG_ANONYMIZER_REGEX
                        .replace_all(inputlog_part_string, |_: &regex::Captures| {
                            format!("\"name\":\"{}\",", p2_anon)
                        });
                    return Some(json!(res));
                } else if inputlog_part_string.starts_with(">chat ") {
                    return None;
                }

                Some(inputlog_part.clone())
            })
            .collect::<Vec<serde_json::Value>>());

        let log = json["log"]
            .as_array()
            .ok_or(format!("Bad JSON for log {}", json["log"]))?
            .iter();

        let p1regex = Regex::from_str(
            &[
                "\\|p1[ab]?: (",
                &regex::escape(p1),
                "|",
                &regex::escape(&p1_id),
                ")",
            ]
            .join(""),
        )?;
        let p2regex = Regex::from_str(
            &[
                "\\|p2[ab]?: (",
                &regex::escape(p2),
                "|",
                &regex::escape(&p2_id),
                ")",
            ]
            .join(""),
        )?;

        json_result["log"] = serde_json::json!(log
            .filter_map(|log_part| {
                let log_part_string: &str = &match log_part.as_str() {
                    Some(a) => a.to_owned(),
                    None => format!("Bad JSON for logpart {:#?}", log_part),
                };

                // Remove chat and timers (privacy threat)
                if log_part_string.starts_with("|c|")
                    || log_part_string.starts_with("|c:|")
                    || log_part_string.starts_with("|inactive|")
                {
                    return None;
                }

                if log_part_string.starts_with("|j|")
                    || log_part_string.starts_with("|J|")
                    || log_part_string.starts_with("|l|")
                    || log_part_string.starts_with("|L|")
                    || log_part_string.starts_with("|N|")
                    || log_part_string.starts_with("|n|")
                    || log_part_string.starts_with("|win|")
                    || log_part_string.starts_with("|tie|")
                    || log_part_string.starts_with("|-message|")
                    || log_part_string.starts_with("|raw|")
                    || log_part_string.starts_with("|player|")
                {
                    return Some(json!(log_part_string
                        .replace(p1, &p1_anon)
                        .replace(p2, &p2_anon)
                        .replace(&p1_id, &p1_anon)
                        .replace(&p2_id, &p2_anon)));
                }

                return Some(json!(p2regex.replace_all(
                    p1regex
                        .replace_all(log_part_string, &p1_anon as &str)
                        .as_ref(),
                    &p2_anon as &str
                )));
            })
            .collect::<Vec<serde_json::Value>>());

        let result = serde_json::to_string(&json_result)?;

        if self.is_safe
            && (result.contains(p1)
                || result.contains(&p1_id)
                || result.contains(p2)
                || result.contains(&p2_id))
        {
            return Err(BattleToolsError::IncompleteAnonymization(
                json["roomid"].to_string(),
            ));
        }

        let battle_number = {
            let mut tracker = self.state.lock().unwrap();
            tracker.current_battle_number += 1;
            tracker.current_battle_number
        };
        Ok((result, battle_number))
    }
}

#[cfg(test)]
mod unit_tests {
    extern crate test;
    use super::*;
    use lazy_static::lazy_static;
    use test::Bencher;

    lazy_static! {
        static ref SAMPLE_JSON: String = String::from(
            r#"{"winner":"Annika","seed":[1,1,1,1],"turns":2,"p1":"Annika","p2":"Rust Haters","p1team":[{"name":"Rotom","species":"Rotom-Fan","gender":"N","shiny":false,"gigantamax":false,"level":84,"moves":["airslash","voltswitch","willowisp","thunderbolt"],"ability":"Levitate","evs":{"hp":85,"atk":0,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":0,"def":31,"spa":31,"spd":31,"spe":31},"item":"Heavy-Duty Boots"},{"name":"Regirock","species":"Regirock","gender":"N","shiny":false,"gigantamax":false,"level":85,"moves":["curse","rockslide","rest","bodypress"],"ability":"Sturdy","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Chesto Berry"},{"name":"Conkeldurr","species":"Conkeldurr","gender":"","shiny":false,"gigantamax":false,"level":80,"moves":["facade","knockoff","machpunch","drainpunch"],"ability":"Guts","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Flame Orb"},{"name":"Reuniclus","species":"Reuniclus","gender":"","shiny":false,"gigantamax":false,"level":84,"moves":["trickroom","focusblast","psychic","shadowball"],"ability":"Magic Guard","evs":{"hp":85,"atk":0,"def":85,"spa":85,"spd":85,"spe":0},"ivs":{"hp":31,"atk":0,"def":31,"spa":31,"spd":31,"spe":0},"item":"Life Orb"},{"name":"Incineroar","species":"Incineroar","gender":"","shiny":false,"gigantamax":false,"level":80,"moves":["knockoff","uturn","earthquake","flareblitz"],"ability":"Intimidate","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Choice Scarf"},{"name":"Miltank","species":"Miltank","gender":"F","shiny":false,"gigantamax":false,"level":84,"moves":["healbell","bodyslam","earthquake","milkdrink"],"ability":"Sap Sipper","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Leftovers"}],"p2team":[{"name":"Drednaw","species":"Drednaw","gender":"","shiny":false,"gigantamax":false,"level":84,"moves":["stoneedge","swordsdance","superpower","liquidation"],"ability":"Swift Swim","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Life Orb"},{"name":"Pinsir","species":"Pinsir","gender":"","shiny":false,"gigantamax":false,"level":84,"moves":["closecombat","stoneedge","xscissor","knockoff"],"ability":"Moxie","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Choice Scarf"},{"name":"Pikachu","species":"Pikachu-Sinnoh","gender":"","shiny":false,"gigantamax":false,"level":92,"moves":["knockoff","volttackle","voltswitch","irontail"],"ability":"Lightning Rod","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Light Ball"},{"name":"Latios","species":"Latios","gender":"M","shiny":false,"gigantamax":false,"level":78,"moves":["dracometeor","calmmind","psyshock","roost"],"ability":"Levitate","evs":{"hp":85,"atk":0,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":0,"def":31,"spa":31,"spd":31,"spe":31},"item":"Soul Dew"},{"name":"Entei","species":"Entei","gender":"N","shiny":false,"gigantamax":false,"level":78,"moves":["flareblitz","stoneedge","extremespeed","sacredfire"],"ability":"Inner Focus","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Choice Band"},{"name":"Exeggutor","species":"Exeggutor-Alola","gender":"","shiny":false,"gigantamax":false,"level":86,"moves":["gigadrain","flamethrower","dracometeor","leafstorm"],"ability":"Frisk","evs":{"hp":85,"atk":0,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":0,"def":31,"spa":31,"spd":31,"spe":31},"item":"Choice Specs"}],"score":[0,2],"inputLog":[">lol you thought i'd leak someone's real input log"],"log":["|j|☆Annika","|j|☆Rust Hater","|player|p1|Annika|cynthia|1400","|player|p2|Rust Hater|cynthia|1100","|teamsize|p1|6","|teamsize|p2|6","|gametype|singles","|gen|8","|tier|[Gen 8] Random Battle","|rated|"],"p1rating":{"entryid":"75790599","userid":"annika","w":"4","l":4,"t":"0","gxe":46.8,"r":1516.9377700433,"rd":121.36211247153,"rptime":1632906000,"rpr":1474.7452159936,"rprd":115.09180605287,"elo":1400.4859871929,"col1":8,"oldelo":"1057.7590112468"},"p2rating":{"entryid":"75790599","userid":"rusthater","w":"4","l":5,"t":"0","gxe":41.8,"r":"1516.9377700433","rd":"121.36211247153","rptime":"1632906000","rpr":1434.9434039083,"rprd":109.84367373045,"elo":1130.7522733629,"col1":9,"oldelo":"1040.4859871929"},"endType":"normal","timestamp":"Wed Nov 1 1970 00:00:01 GMT-0400 (Eastern Daylight Time)","roomid":"battle-gen8randombattle-1","format":"gen8randombattle", "comment": "if you're curious - this is my own rating info & teams from my battles - no violation of privacy here!"}"#
        );
    }

    #[bench]
    pub fn bench_anonymize_unsafe(b: &mut Bencher) {
        let anonymizer = Anonymizer::new(false);
        b.iter(|| anonymizer.anonymize(&SAMPLE_JSON).unwrap());
    }

    #[bench]
    pub fn bench_anonymize_safe(b: &mut Bencher) {
        let anonymizer = Anonymizer::new(true);
        b.iter(|| anonymizer.anonymize(&SAMPLE_JSON).unwrap());
    }

    #[test]
    pub fn player_tracking() {
        let mut state = SharedState::new();
        let player1 = String::from("Annika");
        let player2 = String::from("Annika Testing");

        let anon_p1 = state.anonymize_player(player1.clone());
        let anon_p2 = state.anonymize_player(player2.clone());

        // Unique
        assert_ne!(anon_p1, anon_p2);

        // Anonymous
        assert_ne!(anon_p1, player1);
        assert_ne!(anon_p2, player2);

        // Consistent
        assert_eq!(state.anonymize_player(player1), anon_p1);
        assert_eq!(state.anonymize_player(player2), anon_p2);
    }

    #[test]
    pub fn anonymization() {
        let anonymizer = Anonymizer::new(true);
        let (json, _) = anonymizer.anonymize(&SAMPLE_JSON).unwrap();

        assert_ne!(json, *SAMPLE_JSON);

        for term in ["00:00:01", "Annika", "annika", "Rust Haters", "rusthaters"] {
            assert!(
                !json.contains(term),
                "Identifying information in anonymized JSON ('{}' in '{}')",
                term,
                json
            );
        }

        for property in ["p1rating", "p2rating", "roomid"] {
            let value = gjson::get(&json, property);
            assert!(
                !value.exists() || value.kind() == gjson::Kind::Null,
                "Anonymized JSON includes potentially-identifying property '{}' (full JSON: '{}')",
                property,
                json
            );
        }
    }
}

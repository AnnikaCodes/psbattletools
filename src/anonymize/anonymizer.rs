// From https://github.com/AnnikaCodes/anonbattle/blob/main/src/anonymizer.rs

use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Mutex;

use lazy_static::*;
use regex::Regex;

use crate::{
    id::{escape, to_id},
    BattleToolsError,
};

lazy_static! {
    static ref INPUTLOG_ANONYMIZER_REGEX: Regex = Regex::new(r#"name":".*","#).unwrap();
}

/// Tracks players
#[derive(Serialize, Deserialize)]
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

    fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self)
    }
}

/// Anonymizes string JSON while tracking state
pub struct Anonymizer {
    state: Mutex<SharedState>,
    /// Panics if player names sneak past
    is_safe: bool,
    no_log: bool,
}

impl Anonymizer {
    pub fn new(is_safe: bool, no_log: bool) -> Self {
        Self {
            state: Mutex::new(SharedState::new()),
            is_safe,
            no_log,
        }
    }

    pub fn with_json(json: String, is_safe: bool, no_log: bool) -> Self {
        let state: SharedState = serde_json::from_str(&json).unwrap();
        Self {
            state: Mutex::new(state),
            is_safe,
            no_log,
        }
    }

    /// Anonymizes a log.
    ///
    /// Returns a tuple: (json, battle_number, format)
    pub fn anonymize(&self, raw: &str) -> Result<(String, u32, String), BattleToolsError> {
        let json = json::parse(raw)?;

        let p1 = json["p1"]
            .as_str()
            .ok_or(format!("Bad JSON for p1 {}", json["p1"]))?;
        let p2 = json["p2"]
            .as_str()
            .ok_or(format!("Bad JSON for p2 {}", json["p2"]))?;

        let p1_escaped = escape(p1);
        let p2_escaped = escape(p2);

        let p1_id = to_id(p1);
        let p2_id = to_id(p2);

        let winner = json["winner"]
            .as_str()
            .ok_or(format!("Bad JSON for winner {}", json["winner"]))?
            .to_owned();
        // Don't anonymize an empty string (happens in tied battles)
        let should_anonymize_winner = !winner.is_empty();

        let (p1_anon, p2_anon, winner_anon) = {
            let mut tracker = self.state.lock().unwrap();
            (
                tracker.anonymize_player(p1.to_string()),
                tracker.anonymize_player(p2.to_string()),
                if should_anonymize_winner {
                    tracker.anonymize_player(winner)
                } else {
                    String::new()
                },
            )
        };

        let mut json_result = json.clone();
        // Anonymize
        json_result["p1"] = json::from(p1_anon.clone());
        json_result["p2"] = json::from(p2_anon.clone());
        json_result["winner"] = json::from(winner_anon);

        // Does this need to work for multi battles?
        for player_rating in ["p1rating", "p2rating"] {
            // ELO rounded to the nearest 50
            let anon_elo = match json[player_rating]["elo"].to_string().parse::<f64>() {
                Ok(elo) => json::from((elo / 50.0).round() * 50.0),
                Err(_) => json::JsonValue::Null,
            };

            // `rpr` rounded to the nearest 50
            let anon_rpr = match json[player_rating]["rpr"].to_string().parse::<f64>() {
                Ok(elo) => json::from((elo / 50.0).round() * 50.0),
                Err(_) => json::JsonValue::Null,
            };

            // `rprd` rounded to the nearest 10
            let anon_rprd = match json[player_rating]["rprd"].to_string().parse::<f64>() {
                Ok(elo) => json::from((elo / 10.0).round() * 10.0),
                Err(_) => json::JsonValue::Null,
            };

            json_result[player_rating] = json::object! {
                "elo" => anon_elo,
                "rpr" => anon_rpr,
                "rprd" => anon_rprd,
            };
        }

        json_result["roomid"] = json::JsonValue::Null;

        // "Sat Nov 21 2020 17:05:04 GMT-0500 (Eastern Standard Time)" -> "Sat Nov 21 2020 17"
        let mut timestamp = json["timestamp"]
            .as_str()
            .ok_or(format!("Bad JSON for timestamp {}", json["timestamp"]))?
            .split(':')
            .collect::<Vec<&str>>()[0]
            .to_owned();
        timestamp.push_str(":XX");

        json_result["timestamp"] = json::from(timestamp);

        if self.no_log {
            json_result["inputLog"] = json::array!();
        } else {
            json_result["inputLog"] = json::from(
                json["inputLog"]
                    .members()
                    .filter_map(|inputlog_part| {
                        let inputlog_part_string: &str = inputlog_part.as_str().unwrap();
                        if inputlog_part_string.starts_with(">player p1") {
                            Some(format!(">player p1 {{\\\"\\\"name\\\":\\\"{}\\\"}}", p1))
                        } else if inputlog_part_string.starts_with(">player p2") {
                            Some(format!(">player p2 {{\\\"\\\"name\\\":\\\"{}\\\"}}", p2))
                        } else if inputlog_part_string.starts_with(">chat ") {
                            None
                        } else {
                            Some(
                                inputlog_part
                                    .as_str()
                                    .expect("input log contained non-string values")
                                    .to_string(),
                            )
                        }
                    })
                    .collect::<Vec<_>>(),
            );
        }

        let p1regex = Regex::from_str(
            &[
                "\\|p1[ab]?: (",
                &regex::escape(p1),
                "|",
                &regex::escape(&p1_id),
                "|",
                &regex::escape(&p1_escaped),
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
                "|",
                &regex::escape(&p2_escaped),
                ")",
            ]
            .join(""),
        )?;

        if self.no_log {
            json_result["log"] = json::array!();
        } else {
            json_result["log"] = json::from(
                json["log"]
                    .members()
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
                            if log_part_string.contains("'s rating: ") {
                                return None;
                            }

                            return Some(
                                log_part_string
                                    .replace(p1, &p1_anon)
                                    .replace(p2, &p2_anon)
                                    .replace(&p1_id, &p1_anon)
                                    .replace(&p2_id, &p2_anon)
                                    .replace(&p1_escaped, &p1_anon)
                                    .replace(&p2_escaped, &p2_anon),
                            );
                        }

                        return Some(
                            p2regex
                                .replace_all(
                                    p1regex
                                        .replace_all(log_part_string, &p1_anon as &str)
                                        .as_ref(),
                                    &p2_anon as &str,
                                )
                                .to_string(),
                        );
                    })
                    .collect::<Vec<_>>(),
            );
        }

        let result = json::stringify(json_result);

        if self.is_safe
            && (result.contains(p1)
                || result.contains(&p1_id)
                || result.contains(&p1_escaped)
                || result.contains(p2)
                || result.contains(&p2_id)
                || result.contains(&p2_escaped))
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
        Ok((result, battle_number, json["format"].to_string()))
    }

    // TODO: actually save this & load it
    pub fn get_state_json(&self) -> serde_json::Result<String> {
        self.state.lock().unwrap().to_json()
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
            r#"{"winner":"Annika","seed":[1,1,1,1],"turns":2,"p1":"Annika","p2":"Rust Haters","p1team":[{"name":"Rotom","species":"Rotom-Fan","gender":"N","shiny":false,"gigantamax":false,"level":84,"moves":["airslash","voltswitch","willowisp","thunderbolt"],"ability":"Levitate","evs":{"hp":85,"atk":0,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":0,"def":31,"spa":31,"spd":31,"spe":31},"item":"Heavy-Duty Boots"},{"name":"Regirock","species":"Regirock","gender":"N","shiny":false,"gigantamax":false,"level":85,"moves":["curse","rockslide","rest","bodypress"],"ability":"Sturdy","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Chesto Berry"},{"name":"Conkeldurr","species":"Conkeldurr","gender":"","shiny":false,"gigantamax":false,"level":80,"moves":["facade","knockoff","machpunch","drainpunch"],"ability":"Guts","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Flame Orb"},{"name":"Reuniclus","species":"Reuniclus","gender":"","shiny":false,"gigantamax":false,"level":84,"moves":["trickroom","focusblast","psychic","shadowball"],"ability":"Magic Guard","evs":{"hp":85,"atk":0,"def":85,"spa":85,"spd":85,"spe":0},"ivs":{"hp":31,"atk":0,"def":31,"spa":31,"spd":31,"spe":0},"item":"Life Orb"},{"name":"Incineroar","species":"Incineroar","gender":"","shiny":false,"gigantamax":false,"level":80,"moves":["knockoff","uturn","earthquake","flareblitz"],"ability":"Intimidate","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Choice Scarf"},{"name":"Miltank","species":"Miltank","gender":"F","shiny":false,"gigantamax":false,"level":84,"moves":["healbell","bodyslam","earthquake","milkdrink"],"ability":"Sap Sipper","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Leftovers"}],"p2team":[{"name":"Drednaw","species":"Drednaw","gender":"","shiny":false,"gigantamax":false,"level":84,"moves":["stoneedge","swordsdance","superpower","liquidation"],"ability":"Swift Swim","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Life Orb"},{"name":"Pinsir","species":"Pinsir","gender":"","shiny":false,"gigantamax":false,"level":84,"moves":["closecombat","stoneedge","xscissor","knockoff"],"ability":"Moxie","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Choice Scarf"},{"name":"Pikachu","species":"Pikachu-Sinnoh","gender":"","shiny":false,"gigantamax":false,"level":92,"moves":["knockoff","volttackle","voltswitch","irontail"],"ability":"Lightning Rod","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Light Ball"},{"name":"Latios","species":"Latios","gender":"M","shiny":false,"gigantamax":false,"level":78,"moves":["dracometeor","calmmind","psyshock","roost"],"ability":"Levitate","evs":{"hp":85,"atk":0,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":0,"def":31,"spa":31,"spd":31,"spe":31},"item":"Soul Dew"},{"name":"Entei","species":"Entei","gender":"N","shiny":false,"gigantamax":false,"level":78,"moves":["flareblitz","stoneedge","extremespeed","sacredfire"],"ability":"Inner Focus","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Choice Band"},{"name":"Exeggutor","species":"Exeggutor-Alola","gender":"","shiny":false,"gigantamax":false,"level":86,"moves":["gigadrain","flamethrower","dracometeor","leafstorm"],"ability":"Frisk","evs":{"hp":85,"atk":0,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":0,"def":31,"spa":31,"spd":31,"spe":31},"item":"Choice Specs"}],"score":[0,2],"inputLog":[">lol you thought i'd leak someone's real input log"],"log":["|j|☆Annika","|j|☆Rust Hater","|player|p1|Annika|cynthia|1400","|player|p2|Rust Hater|cynthia|1100","|teamsize|p1|6","|teamsize|p2|6","|gametype|singles","|gen|8","|tier|[Gen 8] Random Battle","|rated|"],"p1rating":{"entryid":"75790599","userid":"annika","w":"4","l":4,"t":"0","gxe":46.8,"r":1516.9377700433,"rd":121.36211247153,"rptime":1632906000,"rpr":1474.7452159936,"rprd":115.09180605287,"elo":1400,"col1":8,"oldelo":"1057.7590112468"},"p2rating":{"entryid":"75790599","userid":"rusthater","w":"4","l":5,"t":"0","gxe":41.8,"r":"1516.9377700433","rd":"121.36211247153","rptime":"1632906000","rpr":1434.9434039083,"rprd":109.84367373045,"elo":1130.7522733629,"col1":9,"oldelo":"1040.4859871929"},"endType":"normal","timestamp":"Wed Nov 1 1970 00:00:01 GMT-0400 (Eastern Daylight Time)","roomid":"battle-gen8randombattle-1","format":"gen8randombattle", "comment": "if you're curious - this is my own rating info & teams from my battles - no violation of privacy here!"}"#
        );
        static ref TIE_WINNERSTRING_JSON: String = String::from(
            r#"{"winner":"","seed":[1,1,1,1],"turns":2,"p1":"Annika","p2":"Rust Haters","p1team":[{"name":"Rotom","species":"Rotom-Fan","gender":"N","shiny":false,"gigantamax":false,"level":84,"moves":["airslash","voltswitch","willowisp","thunderbolt"],"ability":"Levitate","evs":{"hp":85,"atk":0,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":0,"def":31,"spa":31,"spd":31,"spe":31},"item":"Heavy-Duty Boots"},{"name":"Regirock","species":"Regirock","gender":"N","shiny":false,"gigantamax":false,"level":85,"moves":["curse","rockslide","rest","bodypress"],"ability":"Sturdy","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Chesto Berry"},{"name":"Conkeldurr","species":"Conkeldurr","gender":"","shiny":false,"gigantamax":false,"level":80,"moves":["facade","knockoff","machpunch","drainpunch"],"ability":"Guts","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Flame Orb"},{"name":"Reuniclus","species":"Reuniclus","gender":"","shiny":false,"gigantamax":false,"level":84,"moves":["trickroom","focusblast","psychic","shadowball"],"ability":"Magic Guard","evs":{"hp":85,"atk":0,"def":85,"spa":85,"spd":85,"spe":0},"ivs":{"hp":31,"atk":0,"def":31,"spa":31,"spd":31,"spe":0},"item":"Life Orb"},{"name":"Incineroar","species":"Incineroar","gender":"","shiny":false,"gigantamax":false,"level":80,"moves":["knockoff","uturn","earthquake","flareblitz"],"ability":"Intimidate","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Choice Scarf"},{"name":"Miltank","species":"Miltank","gender":"F","shiny":false,"gigantamax":false,"level":84,"moves":["healbell","bodyslam","earthquake","milkdrink"],"ability":"Sap Sipper","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Leftovers"}],"p2team":[{"name":"Drednaw","species":"Drednaw","gender":"","shiny":false,"gigantamax":false,"level":84,"moves":["stoneedge","swordsdance","superpower","liquidation"],"ability":"Swift Swim","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Life Orb"},{"name":"Pinsir","species":"Pinsir","gender":"","shiny":false,"gigantamax":false,"level":84,"moves":["closecombat","stoneedge","xscissor","knockoff"],"ability":"Moxie","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Choice Scarf"},{"name":"Pikachu","species":"Pikachu-Sinnoh","gender":"","shiny":false,"gigantamax":false,"level":92,"moves":["knockoff","volttackle","voltswitch","irontail"],"ability":"Lightning Rod","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Light Ball"},{"name":"Latios","species":"Latios","gender":"M","shiny":false,"gigantamax":false,"level":78,"moves":["dracometeor","calmmind","psyshock","roost"],"ability":"Levitate","evs":{"hp":85,"atk":0,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":0,"def":31,"spa":31,"spd":31,"spe":31},"item":"Soul Dew"},{"name":"Entei","species":"Entei","gender":"N","shiny":false,"gigantamax":false,"level":78,"moves":["flareblitz","stoneedge","extremespeed","sacredfire"],"ability":"Inner Focus","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Choice Band"},{"name":"Exeggutor","species":"Exeggutor-Alola","gender":"","shiny":false,"gigantamax":false,"level":86,"moves":["gigadrain","flamethrower","dracometeor","leafstorm"],"ability":"Frisk","evs":{"hp":85,"atk":0,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":0,"def":31,"spa":31,"spd":31,"spe":31},"item":"Choice Specs"}],"score":[0,2],"inputLog":[">lol you thought i'd leak someone's real input log"],"log":["|j|☆Annika","|j|☆Rust Hater","|player|p1|Annika|cynthia|1400","|player|p2|Rust Hater|cynthia|1100","|teamsize|p1|6","|teamsize|p2|6","|gametype|singles","|gen|8","|tier|[Gen 8] Random Battle","|rated|"],"p1rating":{"entryid":"75790599","userid":"annika","w":"4","l":4,"t":"0","gxe":46.8,"r":1516.9377700433,"rd":121.36211247153,"rptime":1632906000,"rpr":1474.7452159936,"rprd":115.09180605287,"elo":1400,"col1":8,"oldelo":"1057.7590112468"},"p2rating":{"entryid":"75790599","userid":"rusthater","w":"4","l":5,"t":"0","gxe":41.8,"r":"1516.9377700433","rd":"121.36211247153","rptime":"1632906000","rpr":1434.9434039083,"rprd":109.84367373045,"elo":1130.7522733629,"col1":9,"oldelo":"1040.4859871929"},"endType":"normal","timestamp":"Wed Nov 1 1970 00:00:01 GMT-0400 (Eastern Daylight Time)","roomid":"battle-gen8randombattle-1","format":"gen8randombattle", "comment": "if you're curious - this is my own rating info & teams from my battles - no violation of privacy here!"}"#
        );
    }

    #[bench]
    pub fn bench_anonymize_unsafe(b: &mut Bencher) {
        let anonymizer = Anonymizer::new(false, false);
        b.iter(|| anonymizer.anonymize(&SAMPLE_JSON).unwrap());
    }

    #[bench]
    pub fn bench_anonymize_safe(b: &mut Bencher) {
        let anonymizer = Anonymizer::new(true, false);
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
        let anonymizer = Anonymizer::new(true, false);
        let (json, _, _) = anonymizer.anonymize(&SAMPLE_JSON).unwrap();
        assert_ne!(json, *SAMPLE_JSON);

        for term in ["00:00:01", "Annika", "annika", "Rust Haters", "rusthaters"] {
            assert!(
                !json.contains(term),
                "Identifying information in anonymized JSON ('{}' in '{}')",
                term,
                json
            );
        }

        for property in ["roomid"] {
            let value = gjson::get(&json, property);
            assert!(
                !value.exists() || value.kind() == gjson::Kind::Null,
                "Anonymized JSON includes potentially-identifying property '{}' (full JSON: '{}')",
                property,
                json
            );
        }
    }

    // In ties, the `winner` property is an empty string.
    // psbattletools should recognize this and not anonymize it.
    #[test]
    pub fn tie() {
        let anonymizer = Anonymizer::new(true, false);
        let (json, _, _) = anonymizer.anonymize(&TIE_WINNERSTRING_JSON).unwrap();
        assert_eq!(
            gjson::get(&json, "winner").to_string(),
            "".to_string(),
            "`winner` property not anonymized to and empty string in a tie"
        );
    }

    #[test]
    pub fn no_log() {
        let anonymizer_logs = Anonymizer::new(false, false);
        let anonymizer_no_logs = Anonymizer::new(false, true);
        let (logs_json, _, _) = anonymizer_logs.anonymize(&SAMPLE_JSON).unwrap();
        let (no_logs_json, _, _) = anonymizer_no_logs.anonymize(&SAMPLE_JSON).unwrap();

        for should_be_arr in ["log", "inputLog"] {
            assert!(
                gjson::get(&no_logs_json, should_be_arr).kind() == gjson::Kind::Array,
                "{} should be an array in the logless JSON",
                should_be_arr
            );

            assert!(
                gjson::get(&logs_json, should_be_arr).kind() == gjson::Kind::Array,
                "{} should be an array in the JSON with logs",
                should_be_arr
            );
        }

        assert_eq!(gjson::get(&no_logs_json, "log").array().len(), 0);
        assert_eq!(gjson::get(&no_logs_json, "inputLog").array().len(), 0);

        assert_ne!(gjson::get(&logs_json, "log").array().len(), 0);
        assert_ne!(gjson::get(&logs_json, "inputLog").array().len(), 0);
    }
}

use mlb_api::boxscore::Player as ApiPlayer;
use mlb_api::live::FullPlayer;

const DEFAULT_NAME: &str = "-";

#[derive(Debug)]
pub struct Player {
    #[allow(dead_code)]
    pub id: u64,
    pub first_name: String,
    pub last_name: String,
    pub boxscore_name: String,
    pub batter_side: String,
    pub pitch_hand: String,
    pub stats: PlayerStats,
}

#[derive(Default, Debug)]
pub struct PlayerStats {
    pub summary: Option<String>,
    pub note: Option<String>,
    pub pitches_thrown: Option<u8>,
    pub strikes: Option<u8>,
    #[allow(dead_code)]
    pub balls: Option<u8>,
}

impl From<&FullPlayer> for Player {
    fn from(player: &FullPlayer) -> Self {
        Self {
            id: player.id,
            first_name: player
                .use_name
                .clone()
                .unwrap_or_else(|| DEFAULT_NAME.to_owned()),
            last_name: player
                .use_last_name
                .clone()
                .unwrap_or_else(|| DEFAULT_NAME.to_owned()),
            boxscore_name: player
                .boxscore_name
                .clone()
                .unwrap_or_else(|| DEFAULT_NAME.to_owned()),
            batter_side: player
                .bat_side
                .as_ref()
                .map(|b| b.code.clone())
                .unwrap_or_default(),
            pitch_hand: player
                .pitch_hand
                .as_ref()
                .map(|b| format!("{}HP", b.code))
                .unwrap_or_default(),
            stats: PlayerStats::default(), // gets set later
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        Self {
            id: 0,
            first_name: DEFAULT_NAME.to_owned(),
            last_name: DEFAULT_NAME.to_owned(),
            boxscore_name: DEFAULT_NAME.to_owned(),
            batter_side: DEFAULT_NAME.to_owned(),
            pitch_hand: DEFAULT_NAME.to_owned(),
            stats: PlayerStats::default(),
        }
    }
}

impl From<&ApiPlayer> for PlayerStats {
    fn from(person: &ApiPlayer) -> Self {
        let is_pitcher = person.position.position_type == "Pitcher";
        if is_pitcher {
            Self {
                summary: person.stats.pitching.summary.clone(),
                note: person.stats.pitching.note.clone(),
                pitches_thrown: person.stats.pitching.pitches_thrown.map(|p| p as u8),
                strikes: person.stats.pitching.strikes.map(|p| p as u8),
                balls: person.stats.pitching.balls.map(|p| p as u8),
            }
        } else {
            Self {
                summary: person.stats.batting.summary.clone(),
                note: person.stats.batting.note.clone(),
                pitches_thrown: None,
                strikes: None,
                balls: None,
            }
        }
    }
}

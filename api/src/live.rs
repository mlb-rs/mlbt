use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveResponse {
    pub game_pk: i64,
    pub link: String,
    pub meta_data: Option<MetaData>,
    pub game_data: Option<GameData>,
    pub live_data: Option<LiveData>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveData {
    // pub plays: Plays,
    pub linescore: Linescore,
    pub boxscore: Boxscore,
    // pub leaders: Leaders,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Linescore {
    pub current_inning: Option<u8>,
    pub current_inning_ordinal: Option<String>,
    pub inning_state: Option<String>,
    pub inning_half: Option<String>,
    pub is_top_inning: Option<bool>,
    pub scheduled_innings: u8,
    pub innings: Vec<Inning>,
    // pub teams:
    // pub defense:
    // pub offense:
    pub balls: Option<u8>,
    pub strikes: Option<u8>,
    pub outs: Option<u8>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Inning {
    pub num: u8,
    pub ordinal_num: String,
    pub home: TeamInningDetail,
    pub away: TeamInningDetail,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamInningDetail {
    pub runs: Option<u8>,
    pub hits: u8,
    pub errors: u8,
    pub left_on_base: u8,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Boxscore {
    // pub teams: Option<BoxscoreTeams>,
    // pub officials: Option<Vec<Official>>,
    pub info: Option<Vec<FieldListElement>>,
    #[serde(rename = "pitchingNotes")]
    pub pitching_notes: Option<Vec<Option<serde_json::Value>>>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct FieldListElement {
    pub label: Option<String>,
    pub value: Option<String>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaData {
    pub wait: i64,
    pub time_stamp: String,
    pub game_events: Vec<String>,
    pub logical_events: Vec<String>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameData {
    pub game: Game,
    // pub datetime: Datetime,
    // pub status: Status,
    pub teams: Teams,
    // pub players: Players,
    // pub venue: Venue,
    // pub weather: Weather,
    // pub game_info: GameInfo,
    // pub review: Review,
    // pub flags: Flags,
    // pub alerts: Vec<::serde_json::Value>,
    // pub probable_pitchers: ProbablePitchers,
    // pub official_scorer: OfficialScorer,
    // pub primary_datacaster: PrimaryDatacaster,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub pk: i64,
    #[serde(rename = "type")]
    pub type_field: String,
    pub double_header: String,
    pub id: String,
    pub gameday_type: String,
    pub tiebreaker: String,
    pub game_number: i64,
    #[serde(rename = "calendarEventID")]
    pub calendar_event_id: String,
    pub season: String,
    pub season_display: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Teams {
    pub away: Team,
    pub home: Team,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub id: i64,
    pub name: String,
    pub link: String,
    pub season: i64,
    // pub venue: Venue,
    // pub spring_venue: SpringVenue,
    pub team_code: String,
    pub file_code: String,
    pub abbreviation: String,
    pub team_name: String,
    pub location_name: String,
    pub first_year_of_play: String,
    // pub league: League,
    // pub division: Division,
    // pub sport: Sport,
    pub short_name: String,
    // pub record: Record,
    // pub spring_league: SpringLeague,
    pub all_star_status: String,
    pub active: bool,
}

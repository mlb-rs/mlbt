use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveResponse {
    pub game_pk: u64,
    pub link: String,
    pub meta_data: MetaData,
    pub game_data: GameData,
    pub live_data: LiveData,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveData {
    pub plays: Plays,
    pub linescore: Linescore,
    pub boxscore: Boxscore,
    // pub leaders: Leaders,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Plays {
    // #[serde(rename = "allPlays")]
    // pub all_plays: Option<Vec<AllPlay>>,
    #[serde(rename = "currentPlay")]
    pub current_play: Option<CurrentPlay>,
    // #[serde(rename = "scoringPlays")]
    // pub scoring_plays: Option<Vec<i64>>,
    // #[serde(rename = "playsByInning")]
    // pub plays_by_inning: Option<Vec<PlaysByInning>>,
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

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Person {
    pub id: u64,
    #[serde(rename = "fullName")]
    pub full_name: String,
    pub link: Option<String>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Matchup {
    pub batter: Person,
    #[serde(rename = "batSide")]
    pub bat_side: Side,
    pub pitcher: Person,
    #[serde(rename = "pitchHand")]
    pub pitch_hand: Side,
    // #[serde(rename = "batterHotColdZones")]
    // pub batter_hot_cold_zones: Option<Vec<Zone>>,
    // #[serde(rename = "pitcherHotColdZones")]
    // pub pitcher_hot_cold_zones: Option<Vec<Option<serde_json::Value>>>,
    #[serde(rename = "batterHotColdZoneStats")]
    pub batter_hot_cold_zone_stats: Option<BatterHotColdZoneStats>,
}
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Side {
    pub code: Option<SideOptions>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum SideOptions {
    R,
    L,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum HalfInning {
    #[serde(rename = "bottom")]
    Bottom,
    #[serde(rename = "top")]
    Top,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Result {
    // #[serde(rename = "type")]
    // pub result_type: Option<ResultType>,
    pub event: Option<String>,
    #[serde(rename = "eventType")]
    pub event_type: Option<String>,
    pub description: Option<String>,
    pub rbi: Option<u64>,
    #[serde(rename = "awayScore")]
    pub away_score: Option<u8>,
    #[serde(rename = "homeScore")]
    pub home_score: Option<u8>,
}
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct CurrentPlay {
    pub result: Result,
    pub count: Count,
    pub matchup: Matchup,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct BatterHotColdZoneStats {
    pub stats: Vec<StatElement>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct StatElement {
    pub splits: Vec<Split>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Split {
    pub stat: SplitStat,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct SplitStat {
    pub name: String,
    pub zones: Vec<Zone>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Zone {
    pub zone: String,
    pub color: String, // this is what I want: "rgba(255, 255, 255, 0.55)" -> need to convert it to a color
    // pub temp: Temp,
    pub value: String,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Count {
    pub balls: u8,
    pub strikes: u8,
    pub outs: u8,
}

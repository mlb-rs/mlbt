use crate::components::schedule::ScheduleRow;
use crate::state::app_settings::AppSettings;
use crate::state::app_state::HomeOrAway;
use std::process::{Command, Stdio};

#[derive(Copy, Clone, Debug)]
pub enum FeedKind {
    Tv,
    Radio,
}

impl FeedKind {
    fn as_str(self) -> &'static str {
        match self {
            FeedKind::Tv => "tv",
            FeedKind::Radio => "radio",
        }
    }
}

pub fn launch_for_selection(
    settings: &AppSettings,
    row: &ScheduleRow,
    active_team: HomeOrAway,
    kind: FeedKind,
) -> Result<String, String> {
    let configured_cmd = match kind {
        FeedKind::Tv => settings.tv_launch_command.as_deref(),
        FeedKind::Radio => settings.radio_launch_command.as_deref(),
    }
    .ok_or_else(|| {
        format!(
            "no {} launch command configured. Set '{}' in config.",
            kind.as_str(),
            match kind {
                FeedKind::Tv => "tv_launch_command",
                FeedKind::Radio => "radio_launch_command",
            }
        )
    })?;

    let selected_team = match active_team {
        HomeOrAway::Home => row.home_team.abbreviation,
        HomeOrAway::Away => row.away_team.abbreviation,
    };

    let cmd = configured_cmd
        .replace("{team}", selected_team)
        .replace("{home}", row.home_team.abbreviation)
        .replace("{away}", row.away_team.abbreviation)
        .replace("{game_id}", &row.game_id.to_string());

    spawn_shell_command(&cmd)?;
    Ok(cmd)
}

#[cfg(windows)]
fn spawn_shell_command(command: &str) -> Result<(), String> {
    Command::new("cmd")
        .args(["/C", command])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|err| format!("failed to launch command '{command}': {err}"))
}

#[cfg(not(windows))]
fn spawn_shell_command(command: &str) -> Result<(), String> {
    Command::new("sh")
        .args(["-c", command])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|err| format!("failed to launch command '{command}': {err}"))
}

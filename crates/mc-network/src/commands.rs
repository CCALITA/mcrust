use std::fmt;

/// Represents a parsed chat command.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Give {
        player: String,
        item: String,
        count: u32,
    },
    Tp {
        player: String,
        x: f64,
        y: f64,
        z: f64,
    },
    TimeSet {
        value: String,
    },
    TimeAdd {
        ticks: i64,
    },
    Weather {
        state: String,
    },
    Gamemode {
        player: String,
        mode: String,
    },
    Kill {
        target: String,
    },
    Say {
        message: String,
    },
    Seed,
    Help {
        command: Option<String>,
    },
    SetSpawn {
        x: f64,
        y: f64,
        z: f64,
    },
    Difficulty {
        level: String,
    },
}

/// Errors that can occur when parsing a command.
#[derive(Debug, Clone, PartialEq)]
pub enum CommandError {
    UnknownCommand(String),
    MissingArgument {
        command: String,
        argument: String,
    },
    InvalidArgument {
        command: String,
        argument: String,
        reason: String,
    },
    PlayerNotFound(String),
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::UnknownCommand(cmd) => write!(f, "Unknown command: /{cmd}"),
            CommandError::MissingArgument { command, argument } => {
                write!(f, "/{command}: missing required argument <{argument}>")
            }
            CommandError::InvalidArgument {
                command,
                argument,
                reason,
            } => write!(f, "/{command}: invalid argument <{argument}>: {reason}"),
            CommandError::PlayerNotFound(name) => write!(f, "Player not found: {name}"),
        }
    }
}

/// The result of executing a command.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
}

impl CommandResult {
    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
        }
    }
}

/// Parse a chat command string into a `Command`.
///
/// The input must start with `/`. Arguments are whitespace-separated.
///
/// # Examples
///
/// ```
/// use mc_network::commands::parse_command;
///
/// let cmd = parse_command("/give Steve diamond 64").unwrap();
/// ```
pub fn parse_command(input: &str) -> Result<Command, CommandError> {
    let input = input.trim();
    if !input.starts_with('/') {
        return Err(CommandError::UnknownCommand(input.to_string()));
    }

    let parts: Vec<&str> = input[1..].splitn(2, char::is_whitespace).collect();
    let cmd_name = parts[0].to_lowercase();
    let args_str = parts.get(1).unwrap_or(&"").trim();

    match cmd_name.as_str() {
        "give" => parse_give(args_str),
        "tp" | "teleport" => parse_tp(args_str),
        "time" => parse_time(args_str),
        "weather" => parse_weather(args_str),
        "gamemode" | "gm" => parse_gamemode(args_str),
        "kill" => parse_kill(args_str),
        "say" => parse_say(args_str),
        "seed" => Ok(Command::Seed),
        "help" | "?" => parse_help(args_str),
        "setspawn" | "setworldspawn" => parse_setspawn(args_str),
        "difficulty" => parse_difficulty(args_str),
        _ => Err(CommandError::UnknownCommand(cmd_name)),
    }
}

/// Return help text for a specific command or a general overview.
pub fn command_help(cmd: Option<&str>) -> String {
    match cmd {
        Some("give") => "/give <player> <item> [count] - Give an item to a player".to_string(),
        Some("tp") | Some("teleport") => {
            "/tp <player> <x> <y> <z> - Teleport a player to coordinates".to_string()
        }
        Some("time") => {
            "/time set <value> | /time add <ticks> - Set or advance the world time".to_string()
        }
        Some("weather") => "/weather <clear|rain|thunder> - Set the weather".to_string(),
        Some("gamemode") | Some("gm") => {
            "/gamemode <player> <survival|creative|adventure|spectator> - Set player game mode"
                .to_string()
        }
        Some("kill") => "/kill <target> - Kill a player or entity".to_string(),
        Some("say") => "/say <message> - Broadcast a message to all players".to_string(),
        Some("seed") => "/seed - Display the world seed".to_string(),
        Some("help") | Some("?") => "/help [command] - Show help for a command".to_string(),
        Some("setspawn") | Some("setworldspawn") => {
            "/setspawn <x> <y> <z> - Set the world spawn point".to_string()
        }
        Some("difficulty") => {
            "/difficulty <peaceful|easy|normal|hard> - Set the game difficulty".to_string()
        }
        Some(unknown) => format!("Unknown command: /{unknown}. Type /help for a list of commands."),
        None => [
            "Available commands:",
            "  /give <player> <item> [count]",
            "  /tp <player> <x> <y> <z>",
            "  /time set <value> | /time add <ticks>",
            "  /weather <clear|rain|thunder>",
            "  /gamemode <player> <mode>",
            "  /kill <target>",
            "  /say <message>",
            "  /seed",
            "  /help [command]",
            "  /setspawn <x> <y> <z>",
            "  /difficulty <level>",
            "",
            "Type /help <command> for details.",
        ]
        .join("\n"),
    }
}

// ---------------------------------------------------------------------------
// Internal parsers
// ---------------------------------------------------------------------------

fn parse_give(args: &str) -> Result<Command, CommandError> {
    let parts: Vec<&str> = args.split_whitespace().collect();

    let player = parts
        .first()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| CommandError::MissingArgument {
            command: "give".to_string(),
            argument: "player".to_string(),
        })?
        .to_string();

    let item = parts
        .get(1)
        .ok_or_else(|| CommandError::MissingArgument {
            command: "give".to_string(),
            argument: "item".to_string(),
        })?
        .to_string();

    let count = match parts.get(2) {
        Some(s) => s
            .parse::<u32>()
            .map_err(|_| CommandError::InvalidArgument {
                command: "give".to_string(),
                argument: "count".to_string(),
                reason: format!("'{s}' is not a valid positive integer"),
            })?,
        None => 1,
    };

    Ok(Command::Give {
        player,
        item,
        count,
    })
}

fn parse_tp(args: &str) -> Result<Command, CommandError> {
    let parts: Vec<&str> = args.split_whitespace().collect();

    let player = parts
        .first()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| CommandError::MissingArgument {
            command: "tp".to_string(),
            argument: "player".to_string(),
        })?
        .to_string();

    let x = parse_coord(&parts, 1, "tp", "x")?;
    let y = parse_coord(&parts, 2, "tp", "y")?;
    let z = parse_coord(&parts, 3, "tp", "z")?;

    Ok(Command::Tp { player, x, y, z })
}

fn parse_time(args: &str) -> Result<Command, CommandError> {
    let parts: Vec<&str> = args.split_whitespace().collect();

    let sub =
        parts
            .first()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| CommandError::MissingArgument {
                command: "time".to_string(),
                argument: "set|add".to_string(),
            })?;

    match sub.to_lowercase().as_str() {
        "set" => {
            let value = parts
                .get(1)
                .ok_or_else(|| CommandError::MissingArgument {
                    command: "time set".to_string(),
                    argument: "value".to_string(),
                })?
                .to_string();
            Ok(Command::TimeSet { value })
        }
        "add" => {
            let ticks_str = parts.get(1).ok_or_else(|| CommandError::MissingArgument {
                command: "time add".to_string(),
                argument: "ticks".to_string(),
            })?;
            let ticks = ticks_str
                .parse::<i64>()
                .map_err(|_| CommandError::InvalidArgument {
                    command: "time add".to_string(),
                    argument: "ticks".to_string(),
                    reason: format!("'{ticks_str}' is not a valid integer"),
                })?;
            Ok(Command::TimeAdd { ticks })
        }
        other => Err(CommandError::InvalidArgument {
            command: "time".to_string(),
            argument: "subcommand".to_string(),
            reason: format!("expected 'set' or 'add', got '{other}'"),
        }),
    }
}

fn parse_weather(args: &str) -> Result<Command, CommandError> {
    let state = args
        .split_whitespace()
        .next()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| CommandError::MissingArgument {
            command: "weather".to_string(),
            argument: "state".to_string(),
        })?;

    let normalized = state.to_lowercase();
    match normalized.as_str() {
        "clear" | "rain" | "thunder" => Ok(Command::Weather { state: normalized }),
        _ => Err(CommandError::InvalidArgument {
            command: "weather".to_string(),
            argument: "state".to_string(),
            reason: format!("expected 'clear', 'rain', or 'thunder', got '{state}'"),
        }),
    }
}

fn parse_gamemode(args: &str) -> Result<Command, CommandError> {
    let parts: Vec<&str> = args.split_whitespace().collect();

    let player = parts
        .first()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| CommandError::MissingArgument {
            command: "gamemode".to_string(),
            argument: "player".to_string(),
        })?
        .to_string();

    let mode_raw = parts.get(1).ok_or_else(|| CommandError::MissingArgument {
        command: "gamemode".to_string(),
        argument: "mode".to_string(),
    })?;

    let mode = normalize_gamemode(mode_raw).ok_or_else(|| CommandError::InvalidArgument {
        command: "gamemode".to_string(),
        argument: "mode".to_string(),
        reason: format!(
            "expected survival|creative|adventure|spectator (or 0-3), got '{mode_raw}'"
        ),
    })?;

    Ok(Command::Gamemode { player, mode })
}

fn parse_kill(args: &str) -> Result<Command, CommandError> {
    let target = args
        .split_whitespace()
        .next()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| CommandError::MissingArgument {
            command: "kill".to_string(),
            argument: "target".to_string(),
        })?
        .to_string();

    Ok(Command::Kill { target })
}

fn parse_say(args: &str) -> Result<Command, CommandError> {
    if args.is_empty() {
        return Err(CommandError::MissingArgument {
            command: "say".to_string(),
            argument: "message".to_string(),
        });
    }
    Ok(Command::Say {
        message: args.to_string(),
    })
}

fn parse_help(args: &str) -> Result<Command, CommandError> {
    let command = args
        .split_whitespace()
        .next()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    Ok(Command::Help { command })
}

fn parse_setspawn(args: &str) -> Result<Command, CommandError> {
    let parts: Vec<&str> = args.split_whitespace().collect();

    let x = parse_coord(&parts, 0, "setspawn", "x")?;
    let y = parse_coord(&parts, 1, "setspawn", "y")?;
    let z = parse_coord(&parts, 2, "setspawn", "z")?;

    Ok(Command::SetSpawn { x, y, z })
}

fn parse_difficulty(args: &str) -> Result<Command, CommandError> {
    let level_raw = args
        .split_whitespace()
        .next()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| CommandError::MissingArgument {
            command: "difficulty".to_string(),
            argument: "level".to_string(),
        })?;

    let level = normalize_difficulty(level_raw).ok_or_else(|| CommandError::InvalidArgument {
        command: "difficulty".to_string(),
        argument: "level".to_string(),
        reason: format!("expected peaceful|easy|normal|hard (or 0-3), got '{level_raw}'"),
    })?;

    Ok(Command::Difficulty { level })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_coord(parts: &[&str], idx: usize, cmd: &str, name: &str) -> Result<f64, CommandError> {
    let s = parts
        .get(idx)
        .ok_or_else(|| CommandError::MissingArgument {
            command: cmd.to_string(),
            argument: name.to_string(),
        })?;
    s.parse::<f64>().map_err(|_| CommandError::InvalidArgument {
        command: cmd.to_string(),
        argument: name.to_string(),
        reason: format!("'{s}' is not a valid number"),
    })
}

fn normalize_gamemode(input: &str) -> Option<String> {
    match input.to_lowercase().as_str() {
        "survival" | "s" | "0" => Some("survival".to_string()),
        "creative" | "c" | "1" => Some("creative".to_string()),
        "adventure" | "a" | "2" => Some("adventure".to_string()),
        "spectator" | "sp" | "3" => Some("spectator".to_string()),
        _ => None,
    }
}

fn normalize_difficulty(input: &str) -> Option<String> {
    match input.to_lowercase().as_str() {
        "peaceful" | "p" | "0" => Some("peaceful".to_string()),
        "easy" | "e" | "1" => Some("easy".to_string()),
        "normal" | "n" | "2" => Some("normal".to_string()),
        "hard" | "h" | "3" => Some("hard".to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // /give
    // -----------------------------------------------------------------------

    #[test]
    fn parses_give_with_count() {
        let cmd = parse_command("/give Steve diamond 64").unwrap();
        assert_eq!(
            cmd,
            Command::Give {
                player: "Steve".to_string(),
                item: "diamond".to_string(),
                count: 64,
            }
        );
    }

    #[test]
    fn parses_give_defaults_count_to_one() {
        let cmd = parse_command("/give Alex iron_ingot").unwrap();
        assert_eq!(
            cmd,
            Command::Give {
                player: "Alex".to_string(),
                item: "iron_ingot".to_string(),
                count: 1,
            }
        );
    }

    #[test]
    fn give_missing_player() {
        let err = parse_command("/give").unwrap_err();
        assert_eq!(
            err,
            CommandError::MissingArgument {
                command: "give".to_string(),
                argument: "player".to_string(),
            }
        );
    }

    #[test]
    fn give_missing_item() {
        let err = parse_command("/give Steve").unwrap_err();
        assert_eq!(
            err,
            CommandError::MissingArgument {
                command: "give".to_string(),
                argument: "item".to_string(),
            }
        );
    }

    #[test]
    fn give_invalid_count() {
        let err = parse_command("/give Steve diamond abc").unwrap_err();
        assert!(matches!(err, CommandError::InvalidArgument { .. }));
    }

    // -----------------------------------------------------------------------
    // /tp
    // -----------------------------------------------------------------------

    #[test]
    fn parses_tp() {
        let cmd = parse_command("/tp Steve 0 64 0").unwrap();
        assert_eq!(
            cmd,
            Command::Tp {
                player: "Steve".to_string(),
                x: 0.0,
                y: 64.0,
                z: 0.0,
            }
        );
    }

    #[test]
    fn parses_teleport_alias() {
        let cmd = parse_command("/teleport Alex 10.5 70 -30.2").unwrap();
        assert_eq!(
            cmd,
            Command::Tp {
                player: "Alex".to_string(),
                x: 10.5,
                y: 70.0,
                z: -30.2,
            }
        );
    }

    #[test]
    fn tp_missing_coords() {
        let err = parse_command("/tp Steve 0 64").unwrap_err();
        assert_eq!(
            err,
            CommandError::MissingArgument {
                command: "tp".to_string(),
                argument: "z".to_string(),
            }
        );
    }

    #[test]
    fn tp_invalid_coord() {
        let err = parse_command("/tp Steve abc 64 0").unwrap_err();
        assert!(matches!(err, CommandError::InvalidArgument { .. }));
    }

    // -----------------------------------------------------------------------
    // /time
    // -----------------------------------------------------------------------

    #[test]
    fn parses_time_set_day() {
        let cmd = parse_command("/time set day").unwrap();
        assert_eq!(
            cmd,
            Command::TimeSet {
                value: "day".to_string()
            }
        );
    }

    #[test]
    fn parses_time_set_numeric() {
        let cmd = parse_command("/time set 6000").unwrap();
        assert_eq!(
            cmd,
            Command::TimeSet {
                value: "6000".to_string()
            }
        );
    }

    #[test]
    fn parses_time_add() {
        let cmd = parse_command("/time add 1000").unwrap();
        assert_eq!(cmd, Command::TimeAdd { ticks: 1000 });
    }

    #[test]
    fn time_add_negative() {
        let cmd = parse_command("/time add -500").unwrap();
        assert_eq!(cmd, Command::TimeAdd { ticks: -500 });
    }

    #[test]
    fn time_add_invalid_ticks() {
        let err = parse_command("/time add abc").unwrap_err();
        assert!(matches!(err, CommandError::InvalidArgument { .. }));
    }

    #[test]
    fn time_missing_subcommand() {
        let err = parse_command("/time").unwrap_err();
        assert!(matches!(err, CommandError::MissingArgument { .. }));
    }

    #[test]
    fn time_invalid_subcommand() {
        let err = parse_command("/time query").unwrap_err();
        assert!(matches!(err, CommandError::InvalidArgument { .. }));
    }

    // -----------------------------------------------------------------------
    // /weather
    // -----------------------------------------------------------------------

    #[test]
    fn parses_weather_clear() {
        let cmd = parse_command("/weather clear").unwrap();
        assert_eq!(
            cmd,
            Command::Weather {
                state: "clear".to_string()
            }
        );
    }

    #[test]
    fn parses_weather_rain() {
        let cmd = parse_command("/weather rain").unwrap();
        assert_eq!(
            cmd,
            Command::Weather {
                state: "rain".to_string()
            }
        );
    }

    #[test]
    fn parses_weather_thunder() {
        let cmd = parse_command("/weather thunder").unwrap();
        assert_eq!(
            cmd,
            Command::Weather {
                state: "thunder".to_string()
            }
        );
    }

    #[test]
    fn weather_invalid_state() {
        let err = parse_command("/weather sunny").unwrap_err();
        assert!(matches!(err, CommandError::InvalidArgument { .. }));
    }

    #[test]
    fn weather_missing_state() {
        let err = parse_command("/weather").unwrap_err();
        assert!(matches!(err, CommandError::MissingArgument { .. }));
    }

    // -----------------------------------------------------------------------
    // /gamemode
    // -----------------------------------------------------------------------

    #[test]
    fn parses_gamemode_by_name() {
        let cmd = parse_command("/gamemode Steve creative").unwrap();
        assert_eq!(
            cmd,
            Command::Gamemode {
                player: "Steve".to_string(),
                mode: "creative".to_string(),
            }
        );
    }

    #[test]
    fn parses_gamemode_by_number() {
        let cmd = parse_command("/gamemode Alex 0").unwrap();
        assert_eq!(
            cmd,
            Command::Gamemode {
                player: "Alex".to_string(),
                mode: "survival".to_string(),
            }
        );
    }

    #[test]
    fn parses_gm_alias() {
        let cmd = parse_command("/gm Steve 3").unwrap();
        assert_eq!(
            cmd,
            Command::Gamemode {
                player: "Steve".to_string(),
                mode: "spectator".to_string(),
            }
        );
    }

    #[test]
    fn gamemode_invalid_mode() {
        let err = parse_command("/gamemode Steve hardcore").unwrap_err();
        assert!(matches!(err, CommandError::InvalidArgument { .. }));
    }

    #[test]
    fn gamemode_missing_mode() {
        let err = parse_command("/gamemode Steve").unwrap_err();
        assert!(matches!(err, CommandError::MissingArgument { .. }));
    }

    // -----------------------------------------------------------------------
    // /kill
    // -----------------------------------------------------------------------

    #[test]
    fn parses_kill() {
        let cmd = parse_command("/kill Steve").unwrap();
        assert_eq!(
            cmd,
            Command::Kill {
                target: "Steve".to_string()
            }
        );
    }

    #[test]
    fn kill_missing_target() {
        let err = parse_command("/kill").unwrap_err();
        assert!(matches!(err, CommandError::MissingArgument { .. }));
    }

    // -----------------------------------------------------------------------
    // /say
    // -----------------------------------------------------------------------

    #[test]
    fn parses_say() {
        let cmd = parse_command("/say Hello world!").unwrap();
        assert_eq!(
            cmd,
            Command::Say {
                message: "Hello world!".to_string()
            }
        );
    }

    #[test]
    fn say_missing_message() {
        let err = parse_command("/say").unwrap_err();
        assert!(matches!(err, CommandError::MissingArgument { .. }));
    }

    // -----------------------------------------------------------------------
    // /seed
    // -----------------------------------------------------------------------

    #[test]
    fn parses_seed() {
        let cmd = parse_command("/seed").unwrap();
        assert_eq!(cmd, Command::Seed);
    }

    // -----------------------------------------------------------------------
    // /help
    // -----------------------------------------------------------------------

    #[test]
    fn parses_help_no_args() {
        let cmd = parse_command("/help").unwrap();
        assert_eq!(cmd, Command::Help { command: None });
    }

    #[test]
    fn parses_help_with_command() {
        let cmd = parse_command("/help give").unwrap();
        assert_eq!(
            cmd,
            Command::Help {
                command: Some("give".to_string())
            }
        );
    }

    #[test]
    fn parses_question_mark_alias() {
        let cmd = parse_command("/?").unwrap();
        assert_eq!(cmd, Command::Help { command: None });
    }

    // -----------------------------------------------------------------------
    // /setspawn
    // -----------------------------------------------------------------------

    #[test]
    fn parses_setspawn() {
        let cmd = parse_command("/setspawn 100 64 -200").unwrap();
        assert_eq!(
            cmd,
            Command::SetSpawn {
                x: 100.0,
                y: 64.0,
                z: -200.0,
            }
        );
    }

    #[test]
    fn parses_setworldspawn_alias() {
        let cmd = parse_command("/setworldspawn 0 65 0").unwrap();
        assert_eq!(
            cmd,
            Command::SetSpawn {
                x: 0.0,
                y: 65.0,
                z: 0.0,
            }
        );
    }

    #[test]
    fn setspawn_missing_z() {
        let err = parse_command("/setspawn 100 64").unwrap_err();
        assert!(matches!(err, CommandError::MissingArgument { .. }));
    }

    // -----------------------------------------------------------------------
    // /difficulty
    // -----------------------------------------------------------------------

    #[test]
    fn parses_difficulty_by_name() {
        let cmd = parse_command("/difficulty hard").unwrap();
        assert_eq!(
            cmd,
            Command::Difficulty {
                level: "hard".to_string()
            }
        );
    }

    #[test]
    fn parses_difficulty_by_number() {
        let cmd = parse_command("/difficulty 0").unwrap();
        assert_eq!(
            cmd,
            Command::Difficulty {
                level: "peaceful".to_string()
            }
        );
    }

    #[test]
    fn difficulty_invalid_level() {
        let err = parse_command("/difficulty extreme").unwrap_err();
        assert!(matches!(err, CommandError::InvalidArgument { .. }));
    }

    #[test]
    fn difficulty_missing_level() {
        let err = parse_command("/difficulty").unwrap_err();
        assert!(matches!(err, CommandError::MissingArgument { .. }));
    }

    // -----------------------------------------------------------------------
    // Edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn unknown_command() {
        let err = parse_command("/fly").unwrap_err();
        assert_eq!(err, CommandError::UnknownCommand("fly".to_string()));
    }

    #[test]
    fn input_without_slash() {
        let err = parse_command("give Steve diamond 64").unwrap_err();
        assert!(matches!(err, CommandError::UnknownCommand(_)));
    }

    #[test]
    fn case_insensitive_command() {
        let cmd = parse_command("/GIVE Steve diamond 64").unwrap();
        assert_eq!(
            cmd,
            Command::Give {
                player: "Steve".to_string(),
                item: "diamond".to_string(),
                count: 64,
            }
        );
    }

    #[test]
    fn leading_trailing_whitespace() {
        let cmd = parse_command("  /seed  ").unwrap();
        assert_eq!(cmd, Command::Seed);
    }

    // -----------------------------------------------------------------------
    // command_help
    // -----------------------------------------------------------------------

    #[test]
    fn help_returns_overview_for_none() {
        let text = command_help(None);
        assert!(text.contains("Available commands:"));
        assert!(text.contains("/give"));
        assert!(text.contains("/tp"));
    }

    #[test]
    fn help_returns_detail_for_known_command() {
        let text = command_help(Some("give"));
        assert!(text.contains("/give"));
        assert!(text.contains("<player>"));
    }

    #[test]
    fn help_returns_unknown_for_bad_command() {
        let text = command_help(Some("fly"));
        assert!(text.contains("Unknown command"));
    }

    // -----------------------------------------------------------------------
    // CommandError Display
    // -----------------------------------------------------------------------

    #[test]
    fn error_display_unknown() {
        let err = CommandError::UnknownCommand("fly".to_string());
        assert_eq!(err.to_string(), "Unknown command: /fly");
    }

    #[test]
    fn error_display_missing_arg() {
        let err = CommandError::MissingArgument {
            command: "give".to_string(),
            argument: "player".to_string(),
        };
        assert_eq!(err.to_string(), "/give: missing required argument <player>");
    }

    #[test]
    fn error_display_invalid_arg() {
        let err = CommandError::InvalidArgument {
            command: "give".to_string(),
            argument: "count".to_string(),
            reason: "not a number".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "/give: invalid argument <count>: not a number"
        );
    }

    #[test]
    fn error_display_player_not_found() {
        let err = CommandError::PlayerNotFound("Herobrine".to_string());
        assert_eq!(err.to_string(), "Player not found: Herobrine");
    }

    // -----------------------------------------------------------------------
    // CommandResult
    // -----------------------------------------------------------------------

    #[test]
    fn command_result_ok() {
        let r = CommandResult::ok("done");
        assert!(r.success);
        assert_eq!(r.message, "done");
    }

    #[test]
    fn command_result_err() {
        let r = CommandResult::err("failed");
        assert!(!r.success);
        assert_eq!(r.message, "failed");
    }
}

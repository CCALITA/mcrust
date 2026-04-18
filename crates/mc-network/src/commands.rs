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
#[path = "commands_tests.rs"]
mod tests;

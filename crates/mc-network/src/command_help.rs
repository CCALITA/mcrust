/// Generate help text for a specific command or a general overview.
///
/// Pass `None` to get the full list of available commands.
/// Pass `Some("give")` (etc.) to get usage details for a single command.
///
/// # Examples
///
/// ```
/// use mc_network::command_help::command_help;
///
/// let overview = command_help(None);
/// assert!(overview.contains("Available commands:"));
///
/// let detail = command_help(Some("give"));
/// assert!(detail.contains("/give"));
/// ```
pub fn command_help(cmd: Option<&str>) -> String {
    match cmd {
        Some(name) => per_command_help(name),
        None => overview_help(),
    }
}

/// Detailed usage string for a single command.
fn per_command_help(name: &str) -> String {
    match name {
        "give" => "/give <player> <item> [count] - Give an item to a player".to_string(),
        "tp" | "teleport" => {
            "/tp <player> <x> <y> <z> - Teleport a player to coordinates".to_string()
        }
        "time" => {
            "/time set <value> | /time add <ticks> - Set or advance the world time".to_string()
        }
        "weather" => "/weather <clear|rain|thunder> - Set the weather".to_string(),
        "gamemode" | "gm" => {
            "/gamemode <player> <survival|creative|adventure|spectator> - Set player game mode"
                .to_string()
        }
        "kill" => "/kill <target> - Kill a player or entity".to_string(),
        "say" => "/say <message> - Broadcast a message to all players".to_string(),
        "seed" => "/seed - Display the world seed".to_string(),
        "help" | "?" => "/help [command] - Show help for a command".to_string(),
        "setspawn" | "setworldspawn" => {
            "/setspawn <x> <y> <z> - Set the world spawn point".to_string()
        }
        "difficulty" => {
            "/difficulty <peaceful|easy|normal|hard> - Set the game difficulty".to_string()
        }
        unknown => format!("Unknown command: /{unknown}. Type /help for a list of commands."),
    }
}

/// Multi-line overview listing every available command.
fn overview_help() -> String {
    [
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
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn help_teleport_alias() {
        let text = command_help(Some("teleport"));
        assert!(text.contains("/tp"));
    }

    #[test]
    fn help_gm_alias() {
        let text = command_help(Some("gm"));
        assert!(text.contains("/gamemode"));
    }

    #[test]
    fn help_setworldspawn_alias() {
        let text = command_help(Some("setworldspawn"));
        assert!(text.contains("/setspawn"));
    }

    #[test]
    fn help_question_mark_alias() {
        let text = command_help(Some("?"));
        assert!(text.contains("/help"));
    }

    #[test]
    fn overview_contains_all_commands() {
        let text = command_help(None);
        for cmd in &[
            "/give",
            "/tp",
            "/time",
            "/weather",
            "/gamemode",
            "/kill",
            "/say",
            "/seed",
            "/help",
            "/setspawn",
            "/difficulty",
        ] {
            assert!(text.contains(cmd), "overview missing {cmd}");
        }
    }
}

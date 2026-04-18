use super::*;

// /give

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

// /tp

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

// /time

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

// /weather

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

// /gamemode

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

// /kill

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

// /say

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

// /seed

#[test]
fn parses_seed() {
    let cmd = parse_command("/seed").unwrap();
    assert_eq!(cmd, Command::Seed);
}

// /help

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

// /setspawn

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

// /difficulty

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

// Edge cases

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

// CommandError Display

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

// CommandResult

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

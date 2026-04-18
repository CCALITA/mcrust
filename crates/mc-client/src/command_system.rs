use mc_network::command_help::command_help;
use mc_network::commands::{Command, CommandError, parse_command};

// ---------------------------------------------------------------------------
// CommandAction — client-side representation of parsed commands
// ---------------------------------------------------------------------------

/// Actions the client can take after parsing a chat command.
#[derive(Debug, Clone, PartialEq)]
pub enum CommandAction {
    GiveItem(String, u16, u32),
    Teleport(f32, f32, f32),
    SetTime(String),
    SetWeather(String),
    SetGamemode(String),
    KillPlayer,
    ShowSeed,
    ShowHelp(String),
    ChatMessage(String),
    ParseError(String),
}

// ---------------------------------------------------------------------------
// ChatState — in-game chat/command input
// ---------------------------------------------------------------------------

/// Tracks the state of the in-game chat overlay, including the current input
/// buffer and command history.
pub struct ChatState {
    active: bool,
    input_buffer: String,
    history: Vec<String>,
    max_history: usize,
}

impl ChatState {
    pub fn new() -> Self {
        Self {
            active: false,
            input_buffer: String::new(),
            history: Vec::new(),
            max_history: 100,
        }
    }

    /// Activate the chat input overlay.
    pub fn open(&mut self) {
        self.active = true;
    }

    /// Deactivate chat and clear the current input buffer.
    pub fn close(&mut self) {
        self.active = false;
        self.input_buffer.clear();
    }

    /// Append a character to the input buffer.
    pub fn type_char(&mut self, c: char) {
        self.input_buffer.push(c);
    }

    /// Remove the last character from the input buffer.
    pub fn backspace(&mut self) {
        self.input_buffer.pop();
    }

    /// Parse the current input buffer, add it to history, clear the buffer,
    /// and return the resulting [`CommandAction`].
    ///
    /// Returns `None` when the buffer is empty.
    pub fn submit(&mut self) -> Option<CommandAction> {
        let input = self.input_buffer.trim().to_string();
        if input.is_empty() {
            return None;
        }

        // Record in history (cap at max_history)
        self.history.push(input.clone());
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }

        self.input_buffer.clear();

        Some(process_command(&input))
    }

    /// Whether the chat overlay is currently active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// The current (possibly partial) input text.
    pub fn current_input(&self) -> &str {
        &self.input_buffer
    }
}

// ---------------------------------------------------------------------------
// Command processing bridge
// ---------------------------------------------------------------------------

/// Parse a raw input string and map the result to a [`CommandAction`].
///
/// If the input starts with `/`, it is treated as a slash-command and
/// forwarded to [`mc_network::commands::parse_command`]. Otherwise it is
/// returned as a [`CommandAction::ChatMessage`].
pub fn process_command(input: &str) -> CommandAction {
    if !input.starts_with('/') {
        return CommandAction::ChatMessage(input.to_string());
    }

    match parse_command(input) {
        Ok(cmd) => map_command(cmd),
        Err(err) => CommandAction::ParseError(format_error(err)),
    }
}

/// Map a successfully parsed [`Command`] to the corresponding
/// [`CommandAction`].
fn map_command(cmd: Command) -> CommandAction {
    match cmd {
        Command::Give { item, count, .. } => {
            // count is u32 in the network layer; clamp to u16 for the
            // client-side slot representation.
            let stack = count.min(u16::MAX as u32) as u16;
            CommandAction::GiveItem(item, stack, count)
        }
        Command::Tp { x, y, z, .. } => CommandAction::Teleport(x as f32, y as f32, z as f32),
        Command::TimeSet { value } => CommandAction::SetTime(value),
        Command::TimeAdd { ticks } => CommandAction::SetTime(ticks.to_string()),
        Command::Weather { state } => CommandAction::SetWeather(state),
        Command::Gamemode { mode, .. } => CommandAction::SetGamemode(mode),
        Command::Kill { .. } => CommandAction::KillPlayer,
        Command::Say { message } => CommandAction::ChatMessage(message),
        Command::Seed => CommandAction::ShowSeed,
        Command::Help { command } => {
            let text = command_help(command.as_deref());
            CommandAction::ShowHelp(text)
        }
        Command::SetSpawn { x, y, z } => {
            CommandAction::ChatMessage(format!("Spawn set to ({x}, {y}, {z})"))
        }
        Command::Difficulty { level } => {
            CommandAction::ChatMessage(format!("Difficulty set to {level}"))
        }
    }
}

/// Convert a [`CommandError`] into a human-readable string.
fn format_error(err: CommandError) -> String {
    err.to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- ChatState ----------------------------------------------------------

    #[test]
    fn new_chat_state_is_inactive() {
        let state = ChatState::new();
        assert!(!state.is_active());
        assert!(state.current_input().is_empty());
    }

    #[test]
    fn open_and_close_toggles_active() {
        let mut state = ChatState::new();
        state.open();
        assert!(state.is_active());
        state.close();
        assert!(!state.is_active());
    }

    #[test]
    fn type_char_appends_to_buffer() {
        let mut state = ChatState::new();
        state.type_char('h');
        state.type_char('i');
        assert_eq!(state.current_input(), "hi");
    }

    #[test]
    fn backspace_removes_last_char() {
        let mut state = ChatState::new();
        state.type_char('a');
        state.type_char('b');
        state.backspace();
        assert_eq!(state.current_input(), "a");
    }

    #[test]
    fn backspace_on_empty_is_noop() {
        let mut state = ChatState::new();
        state.backspace();
        assert!(state.current_input().is_empty());
    }

    #[test]
    fn submit_empty_returns_none() {
        let mut state = ChatState::new();
        assert!(state.submit().is_none());
    }

    #[test]
    fn submit_clears_buffer_and_records_history() {
        let mut state = ChatState::new();
        for c in "hello".chars() {
            state.type_char(c);
        }
        let action = state.submit();
        assert!(action.is_some());
        assert!(state.current_input().is_empty());
        assert_eq!(state.history.len(), 1);
        assert_eq!(state.history[0], "hello");
    }

    #[test]
    fn close_clears_buffer() {
        let mut state = ChatState::new();
        state.open();
        state.type_char('x');
        state.close();
        assert!(state.current_input().is_empty());
    }

    #[test]
    fn history_caps_at_max() {
        let mut state = ChatState::new();
        state.max_history = 3;
        for i in 0..5 {
            for c in format!("msg{i}").chars() {
                state.type_char(c);
            }
            state.submit();
        }
        assert_eq!(state.history.len(), 3);
        assert_eq!(state.history[0], "msg2");
    }

    // -- process_command ----------------------------------------------------

    #[test]
    fn plain_text_becomes_chat_message() {
        let action = process_command("hello world");
        assert_eq!(
            action,
            CommandAction::ChatMessage("hello world".to_string())
        );
    }

    #[test]
    fn give_command_maps_correctly() {
        let action = process_command("/give Steve diamond 64");
        assert_eq!(
            action,
            CommandAction::GiveItem("diamond".to_string(), 64, 64)
        );
    }

    #[test]
    fn tp_command_maps_to_teleport() {
        let action = process_command("/tp Steve 10 64 -20");
        assert_eq!(action, CommandAction::Teleport(10.0, 64.0, -20.0));
    }

    #[test]
    fn time_set_maps_correctly() {
        let action = process_command("/time set day");
        assert_eq!(action, CommandAction::SetTime("day".to_string()));
    }

    #[test]
    fn weather_maps_correctly() {
        let action = process_command("/weather rain");
        assert_eq!(action, CommandAction::SetWeather("rain".to_string()));
    }

    #[test]
    fn gamemode_maps_correctly() {
        let action = process_command("/gamemode Steve creative");
        assert_eq!(action, CommandAction::SetGamemode("creative".to_string()));
    }

    #[test]
    fn kill_maps_to_kill_player() {
        let action = process_command("/kill Steve");
        assert_eq!(action, CommandAction::KillPlayer);
    }

    #[test]
    fn seed_maps_to_show_seed() {
        let action = process_command("/seed");
        assert_eq!(action, CommandAction::ShowSeed);
    }

    #[test]
    fn help_maps_to_show_help() {
        let action = process_command("/help");
        match action {
            CommandAction::ShowHelp(text) => assert!(text.contains("Available commands:")),
            other => panic!("expected ShowHelp, got {other:?}"),
        }
    }

    #[test]
    fn help_specific_command() {
        let action = process_command("/help give");
        match action {
            CommandAction::ShowHelp(text) => assert!(text.contains("/give")),
            other => panic!("expected ShowHelp, got {other:?}"),
        }
    }

    #[test]
    fn unknown_command_returns_parse_error() {
        let action = process_command("/fly");
        match action {
            CommandAction::ParseError(msg) => assert!(msg.contains("Unknown command")),
            other => panic!("expected ParseError, got {other:?}"),
        }
    }

    #[test]
    fn missing_arg_returns_parse_error() {
        let action = process_command("/give");
        match action {
            CommandAction::ParseError(msg) => assert!(msg.contains("missing")),
            other => panic!("expected ParseError, got {other:?}"),
        }
    }

    #[test]
    fn say_maps_to_chat_message() {
        let action = process_command("/say Hello!");
        assert_eq!(action, CommandAction::ChatMessage("Hello!".to_string()));
    }
}

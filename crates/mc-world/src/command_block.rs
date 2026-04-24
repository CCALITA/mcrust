/// Command block types matching Minecraft's three modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandBlockType {
    /// Activates once on rising redstone edge.
    Impulse,
    /// Activates when the previous command block in the chain succeeds.
    Chain,
    /// Activates every tick while powered.
    Repeat,
}

/// Target selectors for command block commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandTarget {
    /// `@s` — the command block itself (or executing entity).
    SelfTarget,
    /// `@p` — nearest player.
    NearestPlayer,
    /// `@a` — all players.
    AllPlayers,
    /// `@e` — all entities.
    AllEntities,
    /// `@r` — a random player.
    RandomPlayer,
}

/// State of a command block in the world.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandBlock {
    /// The command string to execute.
    pub command: String,
    /// Output from the last execution.
    pub last_output: String,
    /// Whether execution requires the previous block to succeed.
    pub conditional: bool,
    /// The type of command block (Impulse, Chain, or Repeat).
    pub mode: CommandBlockType,
    /// Whether the block activates without a redstone signal (always-active).
    pub auto: bool,
    /// Number of times the last command succeeded (used for comparator output).
    pub success_count: u32,
}

impl CommandBlock {
    /// Creates a new command block with the given mode and command.
    pub fn new(mode: CommandBlockType, command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            last_output: String::new(),
            conditional: false,
            mode,
            auto: false,
            success_count: 0,
        }
    }
}

/// Determines whether a command block should execute this tick.
///
/// - **Impulse**: requires a rising redstone edge (powered and not already
///   auto-active). When `auto` is true, it acts as if always receiving a
///   rising edge, so it fires once per activation cycle.
/// - **Chain**: requires the previous command block in the chain to have
///   succeeded (`prev_success`), unless the block is unconditional
///   (`!conditional`). Also needs power or auto-active.
/// - **Repeat**: fires every tick while powered (or auto-active).
pub fn should_execute(
    cb: &CommandBlock,
    redstone_powered: bool,
    prev_success: bool,
) -> bool {
    let powered = redstone_powered || cb.auto;

    match cb.mode {
        CommandBlockType::Impulse => powered,
        CommandBlockType::Chain => {
            if cb.conditional {
                powered && prev_success
            } else {
                powered
            }
        }
        CommandBlockType::Repeat => powered,
    }
}

/// Executes a command block, returning the output string on success or
/// `None` if the command is empty or should not produce output.
///
/// Updates `last_output` and `success_count` on the block.
pub fn execute_command_block(
    cb: &mut CommandBlock,
    redstone_powered: bool,
) -> Option<String> {
    if cb.command.is_empty() {
        cb.success_count = 0;
        cb.last_output = String::new();
        return None;
    }

    if !should_execute(cb, redstone_powered, true) {
        return None;
    }

    let output = format!("Executed: {}", cb.command);
    cb.last_output = output.clone();
    cb.success_count = 1;
    Some(output)
}

/// Parses a target selector string into a [`CommandTarget`].
///
/// Recognised selectors: `@s`, `@p`, `@a`, `@e`, `@r`.
/// Unknown selectors default to [`CommandTarget::SelfTarget`].
pub fn parse_command_target(selector: &str) -> CommandTarget {
    match selector {
        "@s" => CommandTarget::SelfTarget,
        "@p" => CommandTarget::NearestPlayer,
        "@a" => CommandTarget::AllPlayers,
        "@e" => CommandTarget::AllEntities,
        "@r" => CommandTarget::RandomPlayer,
        _ => CommandTarget::SelfTarget,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- CommandBlock::new ---

    #[test]
    fn new_command_block_has_correct_defaults() {
        let cb = CommandBlock::new(CommandBlockType::Impulse, "say hello");
        assert_eq!(cb.command, "say hello");
        assert_eq!(cb.last_output, "");
        assert!(!cb.conditional);
        assert_eq!(cb.mode, CommandBlockType::Impulse);
        assert!(!cb.auto);
        assert_eq!(cb.success_count, 0);
    }

    #[test]
    fn new_chain_command_block() {
        let cb = CommandBlock::new(CommandBlockType::Chain, "tp @p 0 64 0");
        assert_eq!(cb.mode, CommandBlockType::Chain);
        assert_eq!(cb.command, "tp @p 0 64 0");
    }

    #[test]
    fn new_repeat_command_block() {
        let cb = CommandBlock::new(CommandBlockType::Repeat, "effect @a speed");
        assert_eq!(cb.mode, CommandBlockType::Repeat);
        assert_eq!(cb.command, "effect @a speed");
    }

    // --- should_execute: Impulse ---

    #[test]
    fn impulse_executes_when_powered() {
        let cb = CommandBlock::new(CommandBlockType::Impulse, "say hi");
        assert!(should_execute(&cb, true, false));
    }

    #[test]
    fn impulse_does_not_execute_without_power() {
        let cb = CommandBlock::new(CommandBlockType::Impulse, "say hi");
        assert!(!should_execute(&cb, false, false));
    }

    #[test]
    fn impulse_executes_when_auto() {
        let mut cb = CommandBlock::new(CommandBlockType::Impulse, "say hi");
        cb.auto = true;
        assert!(should_execute(&cb, false, false));
    }

    // --- should_execute: Chain ---

    #[test]
    fn chain_unconditional_executes_when_powered() {
        let cb = CommandBlock::new(CommandBlockType::Chain, "say hi");
        assert!(should_execute(&cb, true, false));
    }

    #[test]
    fn chain_conditional_requires_prev_success() {
        let mut cb = CommandBlock::new(CommandBlockType::Chain, "say hi");
        cb.conditional = true;
        assert!(!should_execute(&cb, true, false));
        assert!(should_execute(&cb, true, true));
    }

    #[test]
    fn chain_conditional_needs_power_and_prev_success() {
        let mut cb = CommandBlock::new(CommandBlockType::Chain, "say hi");
        cb.conditional = true;
        assert!(!should_execute(&cb, false, true));
    }

    #[test]
    fn chain_auto_executes_without_redstone() {
        let mut cb = CommandBlock::new(CommandBlockType::Chain, "say hi");
        cb.auto = true;
        assert!(should_execute(&cb, false, false));
    }

    // --- should_execute: Repeat ---

    #[test]
    fn repeat_executes_when_powered() {
        let cb = CommandBlock::new(CommandBlockType::Repeat, "say hi");
        assert!(should_execute(&cb, true, false));
    }

    #[test]
    fn repeat_does_not_execute_without_power() {
        let cb = CommandBlock::new(CommandBlockType::Repeat, "say hi");
        assert!(!should_execute(&cb, false, false));
    }

    #[test]
    fn repeat_executes_when_auto() {
        let mut cb = CommandBlock::new(CommandBlockType::Repeat, "say hi");
        cb.auto = true;
        assert!(should_execute(&cb, false, false));
    }

    // --- execute_command_block ---

    #[test]
    fn execute_returns_output_when_powered() {
        let mut cb = CommandBlock::new(CommandBlockType::Impulse, "say hello");
        let result = execute_command_block(&mut cb, true);
        assert_eq!(result, Some("Executed: say hello".to_string()));
        assert_eq!(cb.last_output, "Executed: say hello");
        assert_eq!(cb.success_count, 1);
    }

    #[test]
    fn execute_returns_none_when_not_powered() {
        let mut cb = CommandBlock::new(CommandBlockType::Impulse, "say hello");
        let result = execute_command_block(&mut cb, false);
        assert_eq!(result, None);
    }

    #[test]
    fn execute_returns_none_for_empty_command() {
        let mut cb = CommandBlock::new(CommandBlockType::Impulse, "");
        let result = execute_command_block(&mut cb, true);
        assert_eq!(result, None);
        assert_eq!(cb.success_count, 0);
        assert_eq!(cb.last_output, "");
    }

    #[test]
    fn execute_updates_last_output() {
        let mut cb = CommandBlock::new(CommandBlockType::Repeat, "give @p diamond");
        let _ = execute_command_block(&mut cb, true);
        assert_eq!(cb.last_output, "Executed: give @p diamond");
    }

    #[test]
    fn execute_auto_fires_without_redstone() {
        let mut cb = CommandBlock::new(CommandBlockType::Repeat, "time set day");
        cb.auto = true;
        let result = execute_command_block(&mut cb, false);
        assert!(result.is_some());
    }

    // --- parse_command_target ---

    #[test]
    fn parse_self_target() {
        assert_eq!(parse_command_target("@s"), CommandTarget::SelfTarget);
    }

    #[test]
    fn parse_nearest_player() {
        assert_eq!(parse_command_target("@p"), CommandTarget::NearestPlayer);
    }

    #[test]
    fn parse_all_players() {
        assert_eq!(parse_command_target("@a"), CommandTarget::AllPlayers);
    }

    #[test]
    fn parse_all_entities() {
        assert_eq!(parse_command_target("@e"), CommandTarget::AllEntities);
    }

    #[test]
    fn parse_random_player() {
        assert_eq!(parse_command_target("@r"), CommandTarget::RandomPlayer);
    }

    #[test]
    fn parse_unknown_selector_defaults_to_self() {
        assert_eq!(parse_command_target("@x"), CommandTarget::SelfTarget);
        assert_eq!(parse_command_target(""), CommandTarget::SelfTarget);
        assert_eq!(parse_command_target("hello"), CommandTarget::SelfTarget);
    }
}

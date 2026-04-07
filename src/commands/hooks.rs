use std::path::Path;

use crate::cli::HooksAction;
use crate::hooks;

pub fn cmd_hooks(root: &Path, action: HooksAction) {
    match action {
        HooksAction::Install {
            claude,
            cursor,
            copilot,
            agents,
            precommit,
            claude_code_hook,
        } => {
            let targets =
                collect_hook_targets(claude, cursor, copilot, agents, precommit, claude_code_hook);
            hooks::cmd_install(root, &targets);
        }
        HooksAction::Uninstall {
            claude,
            cursor,
            copilot,
            agents,
            precommit,
            claude_code_hook,
        } => {
            let targets =
                collect_hook_targets(claude, cursor, copilot, agents, precommit, claude_code_hook);
            hooks::cmd_uninstall(root, &targets);
        }
        HooksAction::Status => hooks::cmd_status(root),
    }
}

fn collect_hook_targets(
    claude: bool,
    cursor: bool,
    copilot: bool,
    agents: bool,
    precommit: bool,
    claude_code_hook: bool,
) -> Vec<hooks::HookTarget> {
    let mut targets = Vec::new();
    if claude {
        targets.push(hooks::HookTarget::Claude);
    }
    if cursor {
        targets.push(hooks::HookTarget::Cursor);
    }
    if copilot {
        targets.push(hooks::HookTarget::Copilot);
    }
    if agents {
        targets.push(hooks::HookTarget::Agents);
    }
    if precommit {
        targets.push(hooks::HookTarget::Precommit);
    }
    if claude_code_hook {
        targets.push(hooks::HookTarget::ClaudeCodeHook);
    }
    // If no specific targets, empty vec means "all"
    targets
}

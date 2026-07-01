use std::path::Path;

use crate::agents;
use crate::cli::AgentsAction;

pub fn cmd_agents(root: &Path, action: AgentsAction) {
    match action {
        AgentsAction::Install {
            claude,
            cursor,
            codex,
            gemini,
        } => {
            let targets = collect_agent_targets(claude, cursor, codex, gemini);
            agents::cmd_install(root, &targets);
        }
        AgentsAction::Uninstall {
            claude,
            cursor,
            codex,
            gemini,
        } => {
            let targets = collect_agent_targets(claude, cursor, codex, gemini);
            agents::cmd_uninstall(root, &targets);
        }
        AgentsAction::Status => agents::cmd_status(root),
    }
}

fn collect_agent_targets(
    claude: bool,
    cursor: bool,
    codex: bool,
    gemini: bool,
) -> Vec<agents::AgentTool> {
    let mut targets = Vec::new();
    if claude {
        targets.push(agents::AgentTool::Claude);
    }
    if cursor {
        targets.push(agents::AgentTool::Cursor);
    }
    if codex {
        targets.push(agents::AgentTool::Codex);
    }
    if gemini {
        targets.push(agents::AgentTool::Gemini);
    }
    // If no specific targets, empty vec means "all"
    targets
}

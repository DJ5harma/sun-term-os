use std::collections::HashMap;

use crate::config::launcher::LauncherConfig;
use crate::machine::applications::ApplicationEntry;

pub fn merge_launcher_config(
    config: &LauncherConfig,
    discovered: Vec<ApplicationEntry>,
) -> Vec<ApplicationEntry> {
    let mut by_id: HashMap<String, ApplicationEntry> = discovered
        .into_iter()
        .map(|entry| (entry.id.clone(), entry))
        .collect();
    let mut merged = Vec::new();

    for favorite in &config.favorites {
        if let Some(entry) = by_id.remove(favorite) {
            merged.push(entry);
        }
    }

    for extra in &config.extra_commands {
        merged.push(ApplicationEntry {
            id: format!("extra:{}", extra.name),
            name: extra.name.clone(),
            detail: "configured command".to_owned(),
            exec: extra.command.clone(),
            launch_in_terminal: extra.in_terminal,
        });
    }

    let mut rest: Vec<ApplicationEntry> = by_id.into_values().collect();
    rest.sort_by_key(|entry| entry.name.to_lowercase());
    merged.extend(rest);
    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::applications::ApplicationEntry;

    #[test]
    fn favorites_and_extras_precede_alphabetical_rest() {
        let config = LauncherConfig {
            favorites: vec!["b".to_owned()],
            extra_commands: vec![crate::config::launcher::LauncherExtraCommand {
                name: "My Tool".to_owned(),
                command: "mytool".to_owned(),
                in_terminal: true,
            }],
        };
        let discovered = vec![
            ApplicationEntry {
                id: "a".to_owned(),
                name: "Alpha".to_owned(),
                detail: String::new(),
                exec: "a".to_owned(),
                launch_in_terminal: false,
            },
            ApplicationEntry {
                id: "b".to_owned(),
                name: "Beta".to_owned(),
                detail: String::new(),
                exec: "b".to_owned(),
                launch_in_terminal: false,
            },
        ];
        let merged = merge_launcher_config(&config, discovered);
        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].id, "b");
        assert_eq!(merged[1].id, "extra:My Tool");
        assert_eq!(merged[2].id, "a");
    }
}

use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use tempfile::Builder;
use toml_edit::{DocumentMut, Item, value};

#[derive(Debug, Parser)]
#[command(name = "mcps", about = "Enable/disable/remove Codex MCP servers")]
struct Cli {
    #[arg(long, global = true)]
    config: Option<PathBuf>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Enable { name: String },
    Disable { name: String },
    #[command(alias = "delete")]
    Remove { name: String },
    List,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ServerStatus {
    name: String,
    enabled: bool,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let config_path = resolve_config_path(cli.config)?;
    let mut doc = load_config(&config_path)?;

    match cli.command {
        Commands::Enable { name } => {
            set_server_enabled(&mut doc, &name, true)?;
            save_config(&config_path, &doc)?;
            println!("{name} enabled");
        }
        Commands::Disable { name } => {
            set_server_enabled(&mut doc, &name, false)?;
            save_config(&config_path, &doc)?;
            println!("{name} disabled");
        }
        Commands::Remove { name } => {
            remove_server(&mut doc, &name)?;
            save_config(&config_path, &doc)?;
            println!("{name} removed");
        }
        Commands::List => {
            let servers = list_servers(&doc)?;
            print_servers(&servers);
        }
    }

    Ok(())
}

fn resolve_config_path(config_override: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = config_override {
        return Ok(path);
    }

    let home = env::var("HOME").context("HOME environment variable is not set")?;
    Ok(PathBuf::from(home).join(".codex/config.toml"))
}

fn load_config(path: &Path) -> Result<DocumentMut> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("failed to read config file at {}", path.display()))?;
    contents
        .parse::<DocumentMut>()
        .with_context(|| format!("failed to parse TOML in {}", path.display()))
}

fn save_config(path: &Path, doc: &DocumentMut) -> Result<()> {
    let parent = path
        .parent()
        .context("config path has no parent directory")?;

    let mut temp_file = Builder::new()
        .prefix(".mcps.")
        .suffix(".tmp")
        .tempfile_in(parent)
        .with_context(|| format!("failed to create temp file in {}", parent.display()))?;

    temp_file
        .write_all(doc.to_string().as_bytes())
        .with_context(|| format!("failed to write temp config file for {}", path.display()))?;
    temp_file
        .flush()
        .with_context(|| format!("failed to flush temp config file for {}", path.display()))?;

    temp_file
        .persist(path)
        .map_err(|e| e.error)
        .with_context(|| format!("failed to replace config file at {}", path.display()))?;

    Ok(())
}

fn set_server_enabled(doc: &mut DocumentMut, server_name: &str, enabled: bool) -> Result<()> {
    let mcp_servers = doc
        .get_mut("mcp_servers")
        .and_then(Item::as_table_like_mut)
        .ok_or_else(|| anyhow::anyhow!("no [mcp_servers] section found in config"))?;

    let server_item = mcp_servers
        .get_mut(server_name)
        .ok_or_else(|| anyhow::anyhow!("MCP server '{server_name}' not found"))?;
    let server_table = server_item
        .as_table_like_mut()
        .ok_or_else(|| anyhow::anyhow!("MCP server '{server_name}' has invalid table format"))?;

    if let Some(existing) = server_table.get("enabled") {
        if existing.as_bool().is_none() {
            bail!("MCP server '{server_name}' has non-boolean 'enabled' value");
        }
    }

    server_table.insert("enabled", value(enabled));
    Ok(())
}

fn remove_server(doc: &mut DocumentMut, server_name: &str) -> Result<()> {
    let mcp_servers = doc
        .get_mut("mcp_servers")
        .and_then(Item::as_table_like_mut)
        .ok_or_else(|| anyhow::anyhow!("no [mcp_servers] section found in config"))?;

    if mcp_servers.remove(server_name).is_none() {
        bail!("MCP server '{server_name}' not found");
    }

    Ok(())
}

fn list_servers(doc: &DocumentMut) -> Result<Vec<ServerStatus>> {
    let Some(mcp_servers) = doc.get("mcp_servers").and_then(Item::as_table_like) else {
        return Ok(Vec::new());
    };

    let mut out = Vec::new();
    for (name, item) in mcp_servers.iter() {
        let server_table = item
            .as_table_like()
            .ok_or_else(|| anyhow::anyhow!("MCP server '{name}' has invalid table format"))?;
        let enabled = match server_table.get("enabled") {
            None => true,
            Some(v) => v.as_bool().ok_or_else(|| {
                anyhow::anyhow!("MCP server '{name}' has non-boolean 'enabled' value")
            })?,
        };
        out.push(ServerStatus {
            name: name.to_string(),
            enabled,
        });
    }

    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

fn print_servers(servers: &[ServerStatus]) {
    if servers.is_empty() {
        println!("No MCP servers configured.");
        return;
    }

    let width = servers
        .iter()
        .map(|s| s.name.len())
        .max()
        .unwrap_or("Name".len())
        .max("Name".len());
    println!("{:<width$}  Status", "Name", width = width);
    for server in servers {
        let state = if server.enabled { "enabled" } else { "disabled" };
        println!("{:<width$}  {}", server.name, state, width = width);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_doc(input: &str) -> DocumentMut {
        input.parse::<DocumentMut>().unwrap()
    }

    #[test]
    fn enable_existing_server_sets_enabled_true() {
        let mut doc = parse_doc(
            r#"
[mcp_servers.xcode]
command = "xcrun"
args = ["mcpbridge"]
"#,
        );

        set_server_enabled(&mut doc, "xcode", true).unwrap();
        assert_eq!(
            doc["mcp_servers"]["xcode"]["enabled"].as_bool(),
            Some(true)
        );
    }

    #[test]
    fn disable_existing_server_sets_enabled_false() {
        let mut doc = parse_doc(
            r#"
[mcp_servers.xcode]
command = "xcrun"
enabled = true
"#,
        );

        set_server_enabled(&mut doc, "xcode", false).unwrap();
        assert_eq!(
            doc["mcp_servers"]["xcode"]["enabled"].as_bool(),
            Some(false)
        );
    }

    #[test]
    fn toggle_missing_server_fails() {
        let mut doc = parse_doc(
            r#"
[mcp_servers.xcode]
command = "xcrun"
"#,
        );

        let err = set_server_enabled(&mut doc, "missing", true).unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn remove_existing_server_removes_only_that_server() {
        let mut doc = parse_doc(
            r#"
[mcp_servers.xcode]
command = "xcrun"

[mcp_servers.figma]
command = "figma"
"#,
        );

        remove_server(&mut doc, "xcode").unwrap();

        assert!(doc["mcp_servers"].get("xcode").is_none());
        assert_eq!(
            doc["mcp_servers"]["figma"]["command"].as_str(),
            Some("figma")
        );
    }

    #[test]
    fn remove_missing_server_fails() {
        let mut doc = parse_doc(
            r#"
[mcp_servers.xcode]
command = "xcrun"
"#,
        );

        let err = remove_server(&mut doc, "missing").unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn remove_without_mcp_servers_section_fails() {
        let mut doc = parse_doc(
            r#"
[profiles.default]
model = "gpt-5"
"#,
        );

        let err = remove_server(&mut doc, "xcode").unwrap_err();
        assert!(err.to_string().contains("no [mcp_servers] section"));
    }

    #[test]
    fn list_defaults_to_enabled_when_key_missing() {
        let doc = parse_doc(
            r#"
[mcp_servers.xcode]
command = "xcrun"
"#,
        );

        let servers = list_servers(&doc).unwrap();
        assert_eq!(
            servers,
            vec![ServerStatus {
                name: "xcode".to_string(),
                enabled: true
            }]
        );
    }

    #[test]
    fn non_boolean_enabled_is_error() {
        let doc = parse_doc(
            r#"
[mcp_servers.xcode]
command = "xcrun"
enabled = "yes"
"#,
        );

        let err = list_servers(&doc).unwrap_err();
        assert!(err.to_string().contains("non-boolean"));
    }
}

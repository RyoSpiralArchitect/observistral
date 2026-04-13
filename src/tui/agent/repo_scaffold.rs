use std::path::{Path, PathBuf};

pub(super) fn repo_root_from_test_cmd(test_cmd: &str) -> Option<String> {
    for segment in test_cmd.split("&&") {
        let trimmed = segment.trim();
        let Some(path) = trimmed.strip_prefix("test -d ") else {
            continue;
        };
        let path = path.trim().trim_matches('\'').trim_matches('"').trim();
        let Some(root) = path.strip_suffix("/.git") else {
            continue;
        };
        let root = root.trim();
        if !root.is_empty() {
            return Some(root.to_string());
        }
    }
    None
}

pub(super) fn required_repo_files_from_test_cmd(test_cmd: &str, repo_root: &str) -> Vec<String> {
    let mut out = Vec::new();
    for segment in test_cmd.split("&&") {
        let trimmed = segment.trim();
        let Some(path) = trimmed.strip_prefix("test -f ") else {
            continue;
        };
        let path = path.trim().trim_matches('\'').trim_matches('"').trim();
        if path.starts_with(&format!("{repo_root}/")) {
            out.push(path.to_string());
        }
    }
    out
}

pub(super) fn resolve_repo_scaffold_path(tool_root: &str, repo_root: &str) -> PathBuf {
    let repo_path = Path::new(repo_root);
    if repo_path.is_absolute() {
        repo_path.to_path_buf()
    } else {
        Path::new(tool_root).join(repo_path)
    }
}

pub(super) fn resolve_repo_file_path(tool_root: &str, repo_file: &str) -> PathBuf {
    let file_path = Path::new(repo_file);
    if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        Path::new(tool_root).join(file_path)
    }
}

pub(super) fn default_repo_gitignore() -> &'static str {
    ".DS_Store\n.env\n.venv/\n__pycache__/\n*.py[cod]\nnode_modules/\ndist/\nbuild/\n.idea/\n.vscode/\n*.log\n"
}

pub(super) fn scaffold_repo_file_content(target: &str) -> String {
    if target.ends_with("/README.md") {
        return default_repo_readme(target);
    }
    if target.ends_with("/Cargo.toml") {
        return default_rust_cargo_toml(target);
    }
    if target.ends_with("/src/lib.rs") {
        return default_rust_maze_lib();
    }
    if target.ends_with("/src/main.rs") {
        return default_rust_maze_main(target);
    }
    let repo_name = repo_name_from_target(target);
    format!("# {repo_name}\n")
}

fn repo_name_from_target(target: &str) -> &str {
    target
        .trim_end_matches('/')
        .rsplit('/')
        .nth(1)
        .filter(|segment| !segment.trim().is_empty())
        .unwrap_or("project")
}

fn repo_root_from_target(target: &str) -> Option<&str> {
    let (parent, _) = target.rsplit_once('/')?;
    if let Some(root) = parent.strip_suffix("/src") {
        return Some(root);
    }
    Some(parent)
}

fn crate_name_from_target(target: &str) -> String {
    let raw = repo_root_from_target(target)
        .and_then(|root| root.rsplit('/').next())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("project");
    let mut out = String::with_capacity(raw.len());
    let mut prev_underscore = false;
    for ch in raw.chars() {
        let mapped = if ch.is_ascii_alphanumeric() {
            ch.to_ascii_lowercase()
        } else {
            '_'
        };
        if mapped == '_' {
            if prev_underscore {
                continue;
            }
            prev_underscore = true;
            out.push('_');
        } else {
            prev_underscore = false;
            out.push(mapped);
        }
    }
    let out = out.trim_matches('_');
    if out.is_empty() {
        "project".to_string()
    } else if out.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        format!("crate_{out}")
    } else {
        out.to_string()
    }
}

fn default_repo_readme(target: &str) -> String {
    let repo_name = repo_name_from_target(target);
    format!("# {repo_name}\n\nTiny scaffolded repository.\n")
}

fn default_rust_cargo_toml(target: &str) -> String {
    let crate_name = crate_name_from_target(target);
    format!(
        "[package]\nname = \"{crate_name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n"
    )
}

fn default_rust_maze_lib() -> String {
    r#"#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub row: i32,
    pub col: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MazeGame {
    width: i32,
    height: i32,
    player: Position,
    exit: Position,
}

impl MazeGame {
    pub fn new() -> Self {
        Self {
            width: 3,
            height: 3,
            player: Position { row: 0, col: 0 },
            exit: Position { row: 2, col: 2 },
        }
    }

    pub fn player(&self) -> Position {
        self.player
    }

    pub fn apply_move(&mut self, movement: Move) {
        let (dr, dc) = match movement {
            Move::Up => (-1, 0),
            Move::Down => (1, 0),
            Move::Left => (0, -1),
            Move::Right => (0, 1),
        };
        self.player.row = (self.player.row + dr).clamp(0, self.height - 1);
        self.player.col = (self.player.col + dc).clamp(0, self.width - 1);
    }

    pub fn reached_exit(&self) -> bool {
        self.player == self.exit
    }

    pub fn render(&self) -> String {
        let mut rows = Vec::new();
        for row in 0..self.height {
            let mut line = String::new();
            for col in 0..self.width {
                let pos = Position { row, col };
                let ch = if pos == self.player {
                    'P'
                } else if pos == self.exit {
                    'E'
                } else {
                    '.'
                };
                line.push(ch);
            }
            rows.push(line);
        }
        rows.join("\n")
    }
}

impl Default for MazeGame {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_stays_in_bounds() {
        let mut game = MazeGame::new();
        game.apply_move(Move::Left);
        game.apply_move(Move::Up);
        assert_eq!(game.player(), Position { row: 0, col: 0 });
    }

    #[test]
    fn player_can_reach_exit() {
        let mut game = MazeGame::new();
        for movement in [Move::Right, Move::Right, Move::Down, Move::Down] {
            game.apply_move(movement);
        }
        assert!(game.reached_exit());
    }
}
"#
    .to_string()
}

fn default_rust_maze_main(target: &str) -> String {
    let crate_name = crate_name_from_target(target);
    format!(
        "use {crate_name}::{{MazeGame, Move}};\n\nfn main() {{\n    let mut game = MazeGame::new();\n    for movement in [Move::Right, Move::Right, Move::Down] {{\n        game.apply_move(movement);\n    }}\n    println!(\"Maze Game\");\n    println!(\"{{}}\", game.render());\n    if game.reached_exit() {{\n        println!(\"Reached the exit!\");\n    }} else {{\n        println!(\"Keep exploring.\");\n    }}\n}}\n"
    )
}

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
    if is_pygame_maze_target(target) && target.ends_with("/game.py") {
        return default_pygame_maze_game();
    }
    if is_pygame_maze_target(target) && target.ends_with("/main.py") {
        return default_pygame_maze_main();
    }
    if is_pygame_maze_target(target) && target.ends_with("/test_game.py") {
        return default_pygame_maze_test();
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
    if is_pygame_maze_target(target) {
        let repo_name = repo_name_from_target(target);
        return format!(
            "# {repo_name}\n\nTiny pygame maze game.\n\n## Controls\n- Arrow keys: move the player\n- R: reset to the start\n- Esc or window close: quit\n\n## Run\n```bash\npython3 main.py\n```\n"
        );
    }
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

fn is_pygame_maze_target(target: &str) -> bool {
    let low = target.to_ascii_lowercase();
    low.contains("pygame") || (low.contains("maze") && low.ends_with(".py"))
}

fn default_pygame_maze_game() -> String {
    r#"import os

os.environ.setdefault("PYGAME_HIDE_SUPPORT_PROMPT", "1")

import pygame

TILE_SIZE = 48
GRID_WIDTH = 6
GRID_HEIGHT = 6


class MazeGame:
    def __init__(self) -> None:
        self.start = (0, 0)
        self.goal = (5, 5)
        self.walls = {(1, 1), (2, 1), (3, 3), (4, 1)}
        self.player = list(self.start)

    def reset(self) -> None:
        self.player = list(self.start)

    def move(self, dx: int, dy: int) -> None:
        next_col = max(0, min(GRID_WIDTH - 1, self.player[0] + dx))
        next_row = max(0, min(GRID_HEIGHT - 1, self.player[1] + dy))
        if (next_col, next_row) not in self.walls:
            self.player[0] = next_col
            self.player[1] = next_row

    def handle_key(self, key: int) -> None:
        mapping = {
            pygame.K_LEFT: (-1, 0),
            pygame.K_RIGHT: (1, 0),
            pygame.K_UP: (0, -1),
            pygame.K_DOWN: (0, 1),
        }
        if key == pygame.K_r:
            self.reset()
            return
        delta = mapping.get(key)
        if delta is not None:
            self.move(*delta)

    def reached_goal(self) -> bool:
        return tuple(self.player) == self.goal

    def draw(self, surface: pygame.Surface) -> None:
        surface.fill((20, 24, 32))
        for row in range(GRID_HEIGHT):
            for col in range(GRID_WIDTH):
                rect = pygame.Rect(
                    col * TILE_SIZE,
                    row * TILE_SIZE,
                    TILE_SIZE,
                    TILE_SIZE,
                )
                color = (45, 50, 66)
                if (col, row) in self.walls:
                    color = (90, 60, 60)
                elif (col, row) == self.goal:
                    color = (64, 144, 88)
                pygame.draw.rect(surface, color, rect)
                pygame.draw.rect(surface, (16, 18, 24), rect, width=2)

        player_rect = pygame.Rect(
            self.player[0] * TILE_SIZE + 10,
            self.player[1] * TILE_SIZE + 10,
            TILE_SIZE - 20,
            TILE_SIZE - 20,
        )
        pygame.draw.rect(surface, (235, 209, 92), player_rect, border_radius=8)
"#
    .to_string()
}

fn default_pygame_maze_main() -> String {
    r#"import os

os.environ.setdefault("PYGAME_HIDE_SUPPORT_PROMPT", "1")

import pygame

from game import GRID_HEIGHT, GRID_WIDTH, TILE_SIZE, MazeGame


def main() -> None:
    pygame.init()
    screen = pygame.display.set_mode((GRID_WIDTH * TILE_SIZE, GRID_HEIGHT * TILE_SIZE))
    pygame.display.set_caption("Maze Game")
    clock = pygame.time.Clock()
    game = MazeGame()
    running = True

    while running:
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                running = False
            elif event.type == pygame.KEYDOWN:
                if event.key == pygame.K_ESCAPE:
                    running = False
                else:
                    game.handle_key(event.key)

        game.draw(screen)
        pygame.display.flip()
        clock.tick(30)

    pygame.quit()


if __name__ == "__main__":
    main()
"#
    .to_string()
}

fn default_pygame_maze_test() -> String {
    r#"import os

os.environ.setdefault("PYGAME_HIDE_SUPPORT_PROMPT", "1")
os.environ.setdefault("SDL_VIDEODRIVER", "dummy")

import unittest

import pygame

from game import GRID_HEIGHT, GRID_WIDTH, TILE_SIZE, MazeGame


class MazeGameTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        pygame.init()

    @classmethod
    def tearDownClass(cls) -> None:
        pygame.quit()

    def test_player_stays_in_bounds(self) -> None:
        game = MazeGame()
        game.move(-1, 0)
        game.move(0, -1)
        self.assertEqual(tuple(game.player), (0, 0))

    def test_player_can_reach_goal(self) -> None:
        game = MazeGame()
        for key in [
            pygame.K_RIGHT,
            pygame.K_RIGHT,
            pygame.K_RIGHT,
            pygame.K_RIGHT,
            pygame.K_RIGHT,
            pygame.K_DOWN,
            pygame.K_DOWN,
            pygame.K_DOWN,
            pygame.K_DOWN,
            pygame.K_DOWN,
        ]:
            game.handle_key(key)
        self.assertTrue(game.reached_goal())

    def test_draw_renders_to_surface(self) -> None:
        game = MazeGame()
        surface = pygame.Surface((GRID_WIDTH * TILE_SIZE, GRID_HEIGHT * TILE_SIZE))
        game.draw(surface)
        self.assertEqual(surface.get_size(), (GRID_WIDTH * TILE_SIZE, GRID_HEIGHT * TILE_SIZE))


if __name__ == "__main__":
    unittest.main()
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::scaffold_repo_file_content;

    #[test]
    fn rust_scaffold_files_include_maze_logic() {
        let cargo = scaffold_repo_file_content("maze_game/Cargo.toml");
        assert!(cargo.contains("name = \"maze_game\""));

        let lib = scaffold_repo_file_content("maze_game/src/lib.rs");
        assert!(lib.contains("pub struct MazeGame"));
        assert!(lib.contains("fn player_can_reach_exit"));

        let main = scaffold_repo_file_content("maze_game/src/main.rs");
        assert!(main.contains("use maze_game::{MazeGame, Move};"));
    }

    #[test]
    fn pygame_scaffold_files_include_controls_and_tests() {
        let readme = scaffold_repo_file_content("maze_game_pygame/README.md");
        assert!(readme.contains("## Controls"));

        let game = scaffold_repo_file_content("maze_game_pygame/game.py");
        assert!(game.contains("class MazeGame"));
        assert!(game.contains("def draw"));

        let main = scaffold_repo_file_content("maze_game_pygame/main.py");
        assert!(main.contains("from game import"));
        assert!(main.contains("pygame.display.set_mode"));

        let test_file = scaffold_repo_file_content("maze_game_pygame/test_game.py");
        assert!(test_file.contains("class MazeGameTests"));
        assert!(test_file.contains("SDL_VIDEODRIVER"));
    }
}

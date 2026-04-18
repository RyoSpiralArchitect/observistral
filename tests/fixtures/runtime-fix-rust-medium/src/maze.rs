#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct Maze {
    tiles: &'static [&'static str],
    player: Point,
    goal: Point,
}

impl Maze {
    pub fn tiny_fixture() -> Self {
        Self {
            tiles: &["#####", "#S#.#", "#..G#", "#####"],
            player: Point { x: 1, y: 1 },
            goal: Point { x: 3, y: 2 },
        }
    }

    pub fn player_position(&self) -> Point {
        self.player
    }

    pub fn goal_position(&self) -> Point {
        self.goal
    }

    pub fn move_player(&mut self, direction: Direction) {
        let candidate = match direction {
            Direction::Up => Point {
                x: self.player.x,
                y: self.player.y.saturating_sub(1),
            },
            Direction::Down => Point {
                x: self.player.x,
                y: self.player.y + 1,
            },
            Direction::Left => Point {
                x: self.player.x.saturating_sub(1),
                y: self.player.y,
            },
            Direction::Right => Point {
                x: self.player.x,
                y: self.player.y + 1,
            },
        };

        if self.tile_at(candidate) != '#' {
            self.player = candidate;
        }
    }

    fn tile_at(&self, point: Point) -> char {
        self.tiles
            .get(point.y)
            .and_then(|row| row.as_bytes().get(point.x))
            .copied()
            .map(char::from)
            .unwrap_or('#')
    }
}

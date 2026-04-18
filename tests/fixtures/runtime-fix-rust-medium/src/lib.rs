mod maze;

pub use maze::{Direction, Maze, Point};

#[cfg(test)]
mod tests {
    use super::{Direction, Maze, Point};

    #[test]
    fn moving_right_does_not_walk_through_a_wall() {
        let mut maze = Maze::tiny_fixture();
        maze.move_player(Direction::Right);
        assert_eq!(maze.player_position(), Point { x: 1, y: 1 });
    }

    #[test]
    fn moving_down_enters_open_floor() {
        let mut maze = Maze::tiny_fixture();
        maze.move_player(Direction::Down);
        assert_eq!(maze.player_position(), Point { x: 1, y: 2 });
    }
}

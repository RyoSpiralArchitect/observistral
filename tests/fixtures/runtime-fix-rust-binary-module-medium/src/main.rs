mod robot;

use robot::Robot;

fn main() {
    println!("{}", Robot::demo().status());
}

#[cfg(test)]
mod tests {
    use super::robot::{Heading, Robot};

    #[test]
    fn turning_right_from_north_points_east() {
        let mut robot = Robot::demo();
        robot.turn_right();
        assert_eq!(robot.heading(), Heading::East);
    }

    #[test]
    fn turning_left_from_north_points_west() {
        let mut robot = Robot::demo();
        robot.turn_left();
        assert_eq!(robot.heading(), Heading::West);
    }
}

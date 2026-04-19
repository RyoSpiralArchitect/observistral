fn main() {
    println!("{}", heading_label(turn_left(Heading::North)));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Heading {
    North,
    East,
    South,
    West,
}

fn turn_left(heading: Heading) -> Heading {
    match heading {
        Heading::North => Heading::East,
        Heading::East => Heading::North,
        Heading::South => Heading::East,
        Heading::West => Heading::South,
    }
}

fn heading_label(heading: Heading) -> &'static str {
    match heading {
        Heading::North => "northbound",
        Heading::East => "eastbound",
        Heading::South => "southbound",
        Heading::West => "westbound",
    }
}

#[cfg(test)]
mod tests {
    use super::{turn_left, Heading};

    #[test]
    fn turning_left_from_north_points_west() {
        assert_eq!(turn_left(Heading::North), Heading::West);
    }

    #[test]
    fn turning_left_from_east_points_north() {
        assert_eq!(turn_left(Heading::East), Heading::North);
    }
}

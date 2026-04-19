#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Heading {
    North,
    East,
    South,
    West,
}

pub struct Robot {
    heading: Heading,
}

impl Robot {
    pub fn demo() -> Self {
        Self {
            heading: Heading::North,
        }
    }

    pub fn heading(&self) -> Heading {
        self.heading
    }

    pub fn status(&self) -> &'static str {
        match self.heading {
            Heading::North => "northbound",
            Heading::East => "eastbound",
            Heading::South => "southbound",
            Heading::West => "westbound",
        }
    }

    pub fn turn_left(&mut self) {
        self.heading = match self.heading {
            Heading::North => Heading::East,
            Heading::East => Heading::North,
            Heading::South => Heading::East,
            Heading::West => Heading::South,
        };
    }

    pub fn turn_right(&mut self) {
        self.heading = match self.heading {
            Heading::North => Heading::East,
            Heading::East => Heading::South,
            Heading::South => Heading::West,
            Heading::West => Heading::North,
        };
    }
}

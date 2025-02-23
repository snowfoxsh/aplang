use std::cell::RefCell;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;
use std::str::FromStr;
use crate::interpreter::Value;
use crate::interpreter::FunctionMap;
use crate::{downcast, std_function};

pub(super) fn std_robot() -> FunctionMap {
    let mut functions = FunctionMap::new();

    std_function!(functions => fn ROBOT_MAP(map_string: Value::String) {
        let Ok(robot) = map_string.parse::<Robot>() else {
            return Ok(Value::Null);
        };

        Ok(Value::NativeObject(Rc::new(RefCell::new(robot))))
    });

    std_function!(functions => fn MOVE_FOWARD(robot: Value::NativeObject<Robot>) {
        downcast!(robot => Robot);

        let Some(res) = robot.move_forward() else {
            panic!("robot attempted to move into a wall");
        };

        Ok(Value::Bool(res))
    });

    std_function!(functions => fn CAN_MOVE(robot: Value::NativeObject<Robot>, direction: Value::String) {
        downcast!(robot => Robot);
        
        let Ok(direction) = direction.parse::<RelativeDirection>() else {
            return Ok(Value::Null)
        };
        
        let can_move = robot.can_move(direction);

        Ok(Value::Bool(can_move))
    });
    
    std_function!(functions => fn MOVE_FORWARD(robot: Value::NativeObject<Robot>) {
        downcast!(robot => Robot);
        
        let Some(result) = robot.move_forward() else {
            panic!("robot attempted to move into a wall");
        };
        
        Ok(Value::Bool(result))
    });

    std_function!(functions => fn ROTATE_LEFT(robot: Value::NativeObject<Robot>) {
        downcast!(robot => Robot);
        
        robot.rotate_left();
        
        Ok(Value::Null)
    });

    std_function!(functions => fn ROTATE_RIGHT(robot: Value::NativeObject<Robot>) {
        downcast!(robot => Robot);
        
        robot.rotate_right();
        
        Ok(Value::Null)
    });

    std_function!(functions => fn FORMAT_ROBOT(robot: Value::NativeObject<Robot>) {
        downcast!(robot => Robot);
        
        Ok(Value::String(format!("{robot}")))
    });
    
    std_function!(functions => fn FORMAT_ROBOT_ASCII(robot: Value::NativeObject<Robot>) {
        downcast!(robot => Robot);
        
        Ok(Value::String(format!("{robot:?}")))
    });
    
    functions
}


#[derive(Copy, Clone, PartialEq, Eq)]
enum AreaCell {
    Wall,
    Goal,
    Space,
    Checkpoint(u8),
}

#[derive(Debug)]
#[repr(u8)]
#[derive(Copy, Clone)]
enum AreaDirection {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

impl From<i8> for AreaDirection {
    fn from(value: i8) -> Self {
        let v = value.rem_euclid(4);  // ensures it's 0..3 even if the value is negative
        match v {
            0 => AreaDirection::North,
            1 => AreaDirection::East,
            2 => AreaDirection::South,
            3 => AreaDirection::West,
            _ => unreachable!(),
        }
    }
}


#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(i8)]
enum RelativeDirection {
    Forward = 0,
    Left = -1,
    Right = 1,
    Backward = 2,
}

struct Robot {
    area: Vec<Vec<AreaCell>>,
    area_size: (usize, usize),
    location: (usize, usize),
    direction: AreaDirection,
    checkpoint_power: u8,
}

impl FromStr for Robot {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines: Vec<&str> = s.lines().collect();

        // find the maximum width among all lines
        let max_width = lines.iter().map(|line| line.len()).max().unwrap_or(0);
        let height = lines.len();

        // prepare to store the parsed grid
        let mut area = Vec::with_capacity(height);

        // only allow exactly one robot
        let mut robot_location: Option<(usize, usize)> = None;
        let mut robot_direction: Option<AreaDirection> = None;

        // parse each line, left to right. pad with spaces if shorter than max_width
        for (y, line) in lines.iter().enumerate() {
            // if line is shorter pad with spaces
            let mut row_chars: Vec<char> = line.chars().collect();
            if row_chars.len() < max_width {
                row_chars.resize(max_width, ' ');
            }

            let mut row_cells = Vec::with_capacity(max_width);

            for (x, ch) in row_chars.into_iter().enumerate() {
                let cell = match ch {
                    '#' | '@' => AreaCell::Wall,
                    '.' | ',' | ' ' => AreaCell::Space,
                    'x' | 'X' => {
                        AreaCell::Goal
                    }
                    // robot markers
                    'n' | 'N' => {
                        if robot_location.is_some() {
                            return Err(());
                        }
                        robot_location = Some((x, y));
                        robot_direction = Some(AreaDirection::North);
                        AreaCell::Space
                    }
                    's' | 'S' => {
                        if robot_location.is_some() {
                            return Err(());
                        }
                        robot_location = Some((x, y));
                        robot_direction = Some(AreaDirection::South);
                        AreaCell::Space
                    }
                    'e' | 'E' => {
                        if robot_location.is_some() {
                            return Err(());
                        }
                        robot_location = Some((x, y));
                        robot_direction = Some(AreaDirection::East);
                        AreaCell::Space
                    }
                    'w' | 'W' => {
                        if robot_location.is_some() {
                            return Err(());
                        }
                        robot_location = Some((x, y));
                        robot_direction = Some(AreaDirection::West);
                        AreaCell::Space
                    }
                    n if n.is_ascii_digit() => {
                        // convert to number
                        let n= n as u8 - b'0';

                        // checkpoint cannot be zero
                        if n == 0 {
                            return Err(());
                        }

                        AreaCell::Checkpoint(n)
                    },
                    _ => {
                        // anything else => error out
                        return Err(());
                    }
                };
                row_cells.push(cell);
            }
            area.push(row_cells);
        }

        // insure exactly one goal and one robot was found
        let location = match robot_location {
            Some(r) => r,
            None => return Err(()),
        };
        let direction = match robot_direction {
            Some(d) => d,
            None => return Err(()),
        };

        Ok(Robot {
            area,
            area_size: (max_width, height),
            location,
            direction,
            checkpoint_power: 1,
        })
    }
}


impl Robot {
    fn can_move(&self, direction: RelativeDirection) -> bool {
        // calculate the direction to check
        let check_direction = (self.direction as i8 + direction as i8).into();

        let check_pos = match check_direction {
            AreaDirection::North => (self.location.1 as isize - 1, self.location.0 as isize),
            AreaDirection::East  => (self.location.1 as isize,     self.location.0 as isize + 1),
            AreaDirection::South => (self.location.1 as isize + 1, self.location.0 as isize),
            AreaDirection::West  => (self.location.1 as isize,     self.location.0 as isize - 1),
        };

        if check_pos < (0, 0) {
            // out of bounds
            return false
        }

        let Some(check_cell) = self.area
            .get(check_pos.0 as usize)
            .and_then(|col| col.get(check_pos.1 as usize)) else {
            // we are out of bounds
            return false
        };

        *check_cell != AreaCell::Wall
    }

    fn move_forward(&mut self) -> Option<bool> {
        if self.can_move(RelativeDirection::Forward) {
            // recalculate the new position
            match self.direction {
                AreaDirection::North => self.location.1 -= 1,
                AreaDirection::East  => self.location.0 += 1,
                AreaDirection::South => self.location.1 += 1,
                AreaDirection::West  => self.location.0 -= 1,
            }

            let move_cell = self.area[self.location.1][self.location.0];

            match move_cell {
                AreaCell::Goal => {
                    // make sure all checkpoints are captured
                    let checkpoint_exists = self.area.iter()
                        .flatten()
                        .any(|cell| matches!(cell, AreaCell::Checkpoint(_)));

                    if !checkpoint_exists {
                        // capture the goal just for good measure
                        self.area[self.location.1][self.location.0] = AreaCell::Space;
                        Some(true)
                    } else {
                        Some(false)
                    }
                },
                AreaCell::Checkpoint(order) => {
                    // check if the checkpoint can be captured yet
                    if self.checkpoint_power >= order {

                        // capture the checkpoint
                        self.area[self.location.1][self.location.0] = AreaCell::Space;

                        if !self.area.iter().flatten().any(|cell| matches!(cell, AreaCell::Checkpoint(p) if *p <= self.checkpoint_power)) {
                            self.checkpoint_power += 1;
                        }
                    }
                    Some(false)
                },
                AreaCell::Space => {
                    Some(false)
                }
                AreaCell::Wall => {
                    panic!("THIS IS A BUG: Moved into a wall")
                }
            }
        } else {
            None
        }
    }
    fn rotate_left(&mut self) {
        self.direction = (self.direction as i8 - 1).into()
    }

    fn rotate_right(&mut self) {
        self.direction = (self.direction as i8 + 1).into()
    }
}

impl Display for Robot {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let (width, height) = self.area_size;

        write!(f, "┌")?;
        for _ in 0..(width * 3) {
            write!(f, "─")?;
        }
        writeln!(f, "─┐")?;

        for y in 0..height {
            write!(f, "│ ")?;
            for x in 0..width {
                if (x, y) == self.location {
                    // Robot’s direction (using Unicode arrows)
                    let dir_char = match self.direction {
                        AreaDirection::North => "▲▲",
                        AreaDirection::East => "►►",
                        AreaDirection::South => "▼▼",
                        AreaDirection::West => "◄◄",
                    };
                    write!(f, "{dir_char}")?;
                } else {
                    match self.area[y][x] {
                        AreaCell::Wall => write!(f, "██")?,
                        AreaCell::Goal => write!(f, "╳╳")?,
                        AreaCell::Space => write!(f, "░░")?,
                        AreaCell::Checkpoint(power) => write!(f, "{power}{power}")?,
                    }
                }

                write!(f, " ")?;
            }
            write!(f, "│")?;
            writeln!(f)?;
        }

        write!(f, "└")?;
        for _ in 0..(width * 3) {
            write!(f, "─")?;
        }
        writeln!(f, "─┘")?;

        Ok(())
    }
}

impl Debug for Robot {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let (width, height) = self.area_size;

        write!(f, "+")?;
        for _ in 0..(width * 3) {
            write!(f, "-")?;
        }
        writeln!(f, "-+")?;

        for y in 0..height {
            write!(f, "| ")?;
            for x in 0..width {
                if (x, y) == self.location {
                    // Robot’s direction (using Unicode arrows)
                    let dir_char = match self.direction {
                        AreaDirection::North => "nn",
                        AreaDirection::East => "ee",
                        AreaDirection::South => "ss",
                        AreaDirection::West => "ww",
                    };
                    write!(f, "{dir_char}")?;
                } else {
                    match self.area[y][x] {
                        AreaCell::Wall => write!(f, "##")?,
                        AreaCell::Goal => write!(f, "XX")?,
                        AreaCell::Space => write!(f, "..")?,
                        AreaCell::Checkpoint(power) => write!(f, "{power}{power}")?,
                    }
                }

                write!(f, " ")?;
            }
            write!(f, "|")?;
            writeln!(f)?;
        }

        write!(f, "+")?;
        for _ in 0..(width * 3) {
            write!(f, "-")?;
        }
        writeln!(f, "-+")?;

        Ok(())
    }
}

impl FromStr for RelativeDirection {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_uppercase().as_str() {
            "LEFT" => Ok(RelativeDirection::Left),
            "RIGHT" => Ok(RelativeDirection::Right),
            "FORWARD" => Ok(RelativeDirection::Forward),
            "BACKWARD" => Ok(RelativeDirection::Backward),
            _ => Err(())
        }
    }
}
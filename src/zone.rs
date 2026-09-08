use crate::direction::Direction;
use crate::geometry::{Coordinate, Size};
use crate::selection::Selection;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Zone {
    pub position: Coordinate,
    pub size: Size,
}

impl Zone {
    /// Split this zone in half.
    pub fn halved(&self) -> Option<Zone> {
        let half_width = if self.size.width > 1 {
            self.size.width / 2
        } else {
            self.size.width
        };
        let half_x = if self.size.width > 1 {
            self.position.x + half_width / 2
        } else {
            self.position.x
        };
        let half_height = if self.size.height > 1 {
            self.size.height / 2
        } else {
            self.size.height
        };
        let half_y = if self.size.height > 1 {
            self.position.y + half_height / 2
        } else {
            self.position.y
        };

        let new_zone = Zone {
            position: Coordinate {
                x: half_x,
                y: half_y,
            },
            size: Size {
                width: half_width,
                height: half_height,
            },
        };

        if new_zone != *self {
            Some(new_zone)
        } else {
            None
        }
    }

    pub fn moved(
        &self,
        direction: Direction,
        display_width: u32,
        display_height: u32,
    ) -> Option<Zone> {
        let mut new_zone = *self;

        match direction {
            Direction::Left => {
                if new_zone.size.width > new_zone.position.x {
                    new_zone.position.x = 0;
                } else {
                    new_zone.position.x -= new_zone.size.width;
                }
            }
            Direction::Right => {
                if new_zone.size.width >= display_width - new_zone.position.x {
                    new_zone.position.x = display_width - new_zone.size.width;
                } else {
                    new_zone.position.x += new_zone.size.width;
                }
            }
            Direction::Up => {
                if new_zone.size.height > new_zone.position.y {
                    new_zone.position.y = 0;
                } else {
                    new_zone.position.y -= new_zone.size.height;
                }
            }
            Direction::Down => {
                if new_zone.size.height >= display_height - new_zone.position.y {
                    new_zone.position.y = display_height - new_zone.size.height;
                } else {
                    new_zone.position.y += new_zone.size.height;
                }
            }
        }

        if *self != new_zone {
            Some(new_zone)
        } else {
            None
        }
    }

    pub fn from_column_line(
        column: u32,
        line: u32,
        display_width: u32,
        display_height: u32,
    ) -> Self {
        let cell_width = display_width / 10;
        let cell_height = display_height / 30;
        Self {
            position: Coordinate {
                x: column * cell_width,
                y: line * cell_height,
            },
            size: Size {
                width: cell_width,
                height: cell_height,
            },
        }
    }

    pub fn from_selection(selection: &Selection, display_width: u32, display_height: u32) -> Self {
        let column = selection.selected_column.unwrap();
        let line = selection.selected_line.unwrap();
        let division = selection.selected_division.unwrap();

        let division_line = division / 10;
        let division_column = division % 10;
        let cell_width = display_width / 10;
        let cell_height = display_height / 30;
        let sub_width = cell_width / 10;
        let sub_height = cell_height / 3;

        Self {
            position: Coordinate {
                x: column * cell_width + division_column * sub_width,
                y: line * cell_height + division_line * sub_height,
            },
            size: Size {
                width: sub_width,
                height: sub_height,
            },
        }
    }
}

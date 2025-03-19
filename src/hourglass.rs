

#[derive(Clone, Copy)]
#[derive(PartialEq)]
enum LayoutCell {
    Empty,
    Wall(char)
}

enum MoveDirection {
    Down,
    Right,
    Left
}


struct Grid<T> {
    width: usize,
    height: usize,
    cells: Box<[T]>
}

impl<T> Grid<T> {
    pub fn new<F: Fn() -> T>(width: usize, height: usize, element_creator: F) -> Grid<T> {
        Grid::<T> {
            width,
            height,
            cells: (0..(width * height))
                .map(|_| element_creator())
                .collect::<Vec<T>>()
                .into_boxed_slice()
        }
    }

    pub fn is_in_bounds(&self, pos: (usize, usize)) -> bool {
        pos.0 < self.width && pos.1 < self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn flip(&mut self) {
        self.cells.reverse();
    }
}

impl<T: Clone> Clone for Grid<T> {
    fn clone(&self) -> Self {
        let new_cells = self.cells.iter()
            .map(|x| x.clone())
            .collect::<Vec<T>>()
            .into_boxed_slice();

        Grid::<T> {
            width: self.width,
            height: self.height,
            cells: new_cells
        }
    }
}

impl<T> std::ops::Index<(usize, usize)> for Grid<T> {
    type Output = T;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        assert!(self.is_in_bounds(index));
        &self.cells[(index.1 * self.width) + index.0]
    }
}

impl<T> std::ops::IndexMut<(usize, usize)> for Grid<T> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        assert!(self.is_in_bounds(index));
        &mut self.cells[(index.1 * self.width) + index.0]
    }
}


pub struct Hourglass {
    layout: Grid<LayoutCell>,
    state: Grid<u8>
}

impl Hourglass {

    pub const MAX_CELL_SAND: u8 = 2;

    pub fn new(width: usize, height: usize) -> Hourglass {
        assert!(width % 2 == 1, "Width must be odd");
        assert!(height > width, "Height must be more than width");

        let mut layout = Grid::<LayoutCell>::new(width, height, || LayoutCell::Empty);
        Self::populate_layout(&mut layout);

        Hourglass {
            layout,
            state: Grid::<u8>::new(width, height, || 0),
        }
    }

    fn populate_layout(layout: &mut Grid::<LayoutCell>) {
        let height = layout.height();
        let width = layout.width();

        let slope_length = width / 2; // Number of lines with one direction of slash
        let straight_length = height / 2 - slope_length; // Number of lines at the top/bottom without slashes

        // Equalses
        for i in 0..width {
            layout[(i, 0)] = LayoutCell::Wall('=');
            layout[(i, height - 1)] = LayoutCell::Wall('=');
        }

        // Pipes
        for i in 1..straight_length {
            layout[(0, i)] = LayoutCell::Wall('|');
            layout[(width - 1, i)] = LayoutCell::Wall('|');
            layout[(0, height - 1 - i)] = LayoutCell::Wall('|');
            layout[(width - 1, height - 1 - i)] = LayoutCell::Wall('|');
        }

        // Slashes
        for i in 0..slope_length {
            layout[(i, straight_length + i)] = LayoutCell::Wall('\\');
            layout[(width - 1 - i, straight_length + i)] = LayoutCell::Wall('/');
            layout[(slope_length - 1 - i, height - straight_length - slope_length + i)] = LayoutCell::Wall('/');
            layout[(slope_length + 1 + i, height - straight_length - slope_length + i)] = LayoutCell::Wall('\\');
        }

        // Middle pipes (only when odd)
        if height % 2 == 1 {
            layout[(width / 2 - 1, height / 2)] = LayoutCell::Wall('|');
            layout[(width / 2 + 1, height / 2)] = LayoutCell::Wall('|');
        }
    }


    pub fn width(&self) -> usize {
        self.layout.width()
    }

    pub fn height(&self) -> usize {
        self.layout.height()
    }


    pub fn is_solid_at(&self, pos: (usize, usize)) -> bool {
        if !self.layout.is_in_bounds(pos) {
            true
        } else {
            match self.layout[pos] {
                LayoutCell::Wall(_) => true,
                _ => self.state[pos] >= Hourglass::MAX_CELL_SAND
            }
        }
    }


    pub fn try_place_sand(&mut self, pos: (usize, usize)) -> bool {
        if self.state[pos] < Hourglass::MAX_CELL_SAND {
            self.state[pos] += 1;
            true
        } else {
            false
        }
    }

    pub fn try_add_sand(&mut self) -> bool {
        self.try_place_sand((self.width() / 2, 1))
    }


    pub fn advance(&mut self, rng: &mut impl rand::Rng) -> usize {
        let mut moves: usize = 0;

        for y in (0..(self.height())).rev() {
            for x in 0..self.width() {
                let here = (x, y);
                assert!(self.state[here] <= Hourglass::MAX_CELL_SAND);

                let dir = match rng.random_range(0..3) {
                    0 => MoveDirection::Down,
                    1 => MoveDirection::Right,
                    2 => MoveDirection::Left,
                    _ => panic!()
                };

                if self.can_flow(&here, &dir) {
                    self.state[here] -= 1;
                    match dir {
                        MoveDirection::Down => self.state[(x, y + 1)] += 1,
                        MoveDirection::Right => self.state[(x + 1, y)] += 1,
                        MoveDirection::Left => self.state[(x - 1, y)] += 1,
                    };
                    moves += 1;
                }
            }
        }

        moves
    }

    pub fn flip(&mut self) {
        self.state.flip();
        self.layout.flip();
    }


    fn can_flow(&self, pos: &(usize, usize), dir: &MoveDirection) -> bool {
        assert!(self.state.is_in_bounds(*pos));

        let solid_below = self.is_solid_at((pos.0, pos.1 + 1));

        let sand_here = self.state[*pos];
        if sand_here < 1 {
            return false;
        }

        match dir {
            MoveDirection::Down => pos.1 < self.height() - 1 && !solid_below,
            MoveDirection::Right => solid_below && !self.is_solid_at((pos.0 + 1, pos.1)) && ((sand_here > 1 && sand_here - 1 > self.state[(pos.0 + 1, pos.1)]) || (!self.is_solid_at((pos.0 + 1, pos.1)) && !self.is_solid_at((pos.0 + 1, pos.1 + 1)))),
            MoveDirection::Left => solid_below && !self.is_solid_at((pos.0 - 1, pos.1)) && ((sand_here > 1 && sand_here - 1 > self.state[(pos.0 - 1, pos.1)]) || (!self.is_solid_at((pos.0 - 1, pos.1)) && !self.is_solid_at((pos.0 - 1, pos.1 + 1)))),
        }
    }

}

impl std::fmt::Display for Hourglass {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.height() {
            for x in 0..self.width() {
                write!(
                    f,
                    "{}",
                    match self.layout[(x, y)] {
                        LayoutCell::Empty => match self.state[(x, y)] {
                            0 => ' ',
                            1 => '.',
                            2 => ':',
                            _ => '?'
                        },
                        LayoutCell::Wall(ch) => ch
                    }
                )?;
            }

            if y < self.height() - 1 { writeln!(f, "")?; }
        }

        Ok(())
    }

}

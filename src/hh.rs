use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use rand::{self, Rng};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::{CrosstermBackend, Terminal},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::io::{self, stdout};

fn main() -> io::Result<()> {
    let mut game = Game::new();
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut should_quit = false;
    while !should_quit {
        terminal.draw(|frame| {
            ui(frame, &game);
        })?;
        should_quit = handle_events(&mut game)?;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

#[derive(Debug, Clone)]
struct Cell {
    is_mine: bool,
    is_revealed: bool,
    is_flagged: bool,
    adjacent_mines: u8,
}

#[derive(Debug, Clone)]
struct Board {
    cells: Vec<Vec<Cell>>,
    mines: i32,
    side: usize,
}

#[derive(Debug, Clone)]
struct Game {
    board: Board,
    is_over: bool,
    is_won: bool,
    selected_cell: (usize, usize),
    flagged: i32,
    first_reveal: bool,
}

impl Game {
    fn new() -> Self {
        let board = logic(None);
        Self {
            board,
            is_over: false,
            is_won: false,
            selected_cell: (0, 0),
            flagged: 0,
            first_reveal: true,
        }
    }
    fn reveal_cell(&mut self, x: usize, y: usize) {
        if self.board.cells[x][y].is_revealed || self.board.cells[x][y].is_flagged {
            return;
        }
        self.board.cells[x][y].is_revealed = true;
        if self.board.cells[x][y].is_mine {
            self.is_over = true;
            return;
        }
        if self.board.cells[x][y].adjacent_mines == 0 {
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let new_x = x as isize + dx;
                    let new_y = y as isize + dy;
                    if new_x >= 0
                        && new_x < self.board.side as isize
                        && new_y >= 0
                        && new_y < self.board.side as isize
                    {
                        self.reveal_cell(new_x as usize, new_y as usize);
                    }
                }
            }
        }
    }

    fn reveal_adjacent_cells(&mut self, x: usize, y: usize) {
        let cell = &self.board.cells[x][y];
        if !cell.is_revealed || cell.adjacent_mines == 0 {
            return;
        }

        let mut flagged_count = 0;
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let new_x = x as isize + dx;
                let new_y = y as isize + dy;
                if new_x >= 0
                    && new_x < self.board.side as isize
                    && new_y >= 0
                    && new_y < self.board.side as isize
                {
                    if self.board.cells[new_x as usize][new_y as usize].is_flagged {
                        flagged_count += 1;
                    }
                }
            }
        }

        if flagged_count == cell.adjacent_mines {
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let new_x = x as isize + dx;
                    let new_y = y as isize + dy;
                    if new_x >= 0
                        && new_x < self.board.side as isize
                        && new_y >= 0
                        && new_y < self.board.side as isize
                    {
                        if !self.board.cells[new_x as usize][new_y as usize].is_flagged {
                            self.reveal_cell(new_x as usize, new_y as usize);
                        }
                    }
                }
            }
        }
    }
}

impl Board {
    fn new(side: usize, mines: i32) -> Self {
        let mut cellsset: Vec<Vec<Cell>> = vec![];
        for _ in 0..side {
            let mut row: Vec<Cell> = vec![];
            for _ in 0..side {
                row.push(Cell {
                    is_mine: false,
                    is_revealed: false,
                    is_flagged: false,
                    adjacent_mines: 0,
                });
            }
            cellsset.push(row);
        }
        Self {
            cells: cellsset,
            mines: mines,
            side: side,
        }
    }

    fn place_mines(&mut self, x: usize, y: usize) {
        let mut mines = self.mines;
        while mines > 0 {
            let i = rand::thread_rng().gen_range(0..self.side);
            let j = rand::thread_rng().gen_range(0..self.side);
            if i == x && j == y {
                continue;
            }
            if self.cells[i][j].is_mine {
                continue;
            }
            self.cells[i][j].is_mine = true;
            mines -= 1;
        }
    }

    fn update_adjacent_mines(&mut self) {
        for i in 0..self.side {
            for j in 0..self.side {
                if self.cells[i][j].is_mine {
                    continue;
                }
                let mut count = 0;
                for x in -1..=1 {
                    for y in -1..=1 {
                        if x == 0 && y == 0 {
                            continue;
                        }
                        let new_x = i as i32 + x;
                        let new_y = j as i32 + y;
                        if new_x >= 0
                            && new_x < self.side as i32
                            && new_y >= 0
                            && new_y < self.side as i32
                        {
                            if self.cells[new_x as usize][new_y as usize].is_mine {
                                count += 1;
                            }
                        }
                    }
                }
                self.cells[i][j].adjacent_mines = count;
            }
        }
    }
}

fn logic(boardopt: Option<Board>) -> Board {
    let side = 9;
    let mines = 10;

    let board = boardopt.unwrap_or(Board::new(side, mines));

    // // placing mines
    // board.place_mines();

    // // update adjacent mines
    // board.update_adjacent_mines();

    // for row in &board.cells {
    //     for cell in row {
    //         if cell.is_mine {
    //             print!("* ");
    //         } else {
    //             print!("{:?} ", cell.adjacent_mines);
    //         }
    //     }
    //     println!();
    // }
    //
    board
}

fn handle_events(game: &mut Game) -> io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(true),
                    KeyCode::Char('f') => {
                        let (x, y) = game.selected_cell;
                        game.board.cells[x][y].is_flagged = !game.board.cells[x][y].is_flagged;
                        if game.board.cells[x][y].is_flagged {
                            game.flagged += 1;
                        } else {
                            game.flagged -= 1;
                        }
                        if game.flagged == game.board.mines {
                            game.is_won = true;
                        }
                    }
                    KeyCode::Char('r') => {
                        let (x, y) = game.selected_cell;
                        // if first reveal, place mines
                        if game.first_reveal {
                            game.first_reveal = false;
                            game.board.place_mines(x, y);
                            game.board.update_adjacent_mines();
                        }
                        if game.board.cells[x][y].is_revealed {
                            game.reveal_adjacent_cells(x, y);
                        } else {
                            game.reveal_cell(x, y);
                        }
                    }
                    KeyCode::Up => {
                        if game.selected_cell.0 > 0 {
                            game.selected_cell.0 -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if game.selected_cell.0 < game.board.side - 1 {
                            game.selected_cell.0 += 1;
                        }
                    }
                    KeyCode::Left => {
                        if game.selected_cell.1 > 0 {
                            game.selected_cell.1 -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if game.selected_cell.1 < game.board.side - 1 {
                            game.selected_cell.1 += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(false)
}

fn centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn ui(frame: &mut Frame, game: &Game) {
    // let mut board_display = String::new();

    // board_display.push_str(format!("Flags: {}\n", 10 - game.flagged).as_str());
    // for (i, row) in game.board.cells.iter().enumerate() {
    //     for (j, cell) in row.iter().enumerate() {
    //         if game.selected_cell == (i, j) {
    //             board_display.push_str("[ ");
    //         } else {
    //             board_display.push_str("  ");
    //         }

    //         if cell.is_revealed {
    //             if cell.is_mine {
    //                 board_display.push('*');
    //             } else {
    //                 board_display.push_str(&cell.adjacent_mines.to_string());
    //             }
    //         } else if cell.is_flagged {
    //             board_display.push('F');
    //         } else {
    //             board_display.push('#');
    //         }

    //         if game.selected_cell == (i, j) {
    //             board_display.push_str(" ]");
    //         } else {
    //             board_display.push_str("  ");
    //         }

    //         board_display.push(' ');
    //     }
    //     board_display.push_str("\n\n");
    // }

    // if game.is_over {
    //     board_display.push_str("Game Over!");
    // }

    // if game.is_won {
    //     board_display.push_str("You won!");
    // }
    //

    // let paragraph = Paragraph::new(board_display)
    //     .block(Block::default().borders(Borders::ALL).title("Minesweeper"))
    //     .centered();
    // frame.render_widget(paragraph, centered_rect(frame.size(), 40, 40));
    //
    // render cells individually
    let cell_width = 3;
    let cell_height = 2;
    let board_width = game.board.side * cell_width;
    let board_height = game.board.side * cell_height;

    let board_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(board_height as u16),
            Constraint::Length(3),
        ])
        .split(centered_rect(
            frame.size(),
            board_width as u16,
            board_height as u16,
        ));

    for (i, row) in game.board.cells.iter().enumerate() {
        let row_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                (0..game.board.side)
                    .map(|_| Constraint::Length(cell_width as u16))
                    .collect::<Vec<_>>(),
            )
            .split(board_layout[0]);

        for (j, cell) in row.iter().enumerate() {
            let cell_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(cell_height as u16),
                    Constraint::Length(1),
                ])
                .split(row_layout[j]);

            let cell_display = if game.selected_cell == (i, j) {
                format!(
                    "[{}]",
                    if cell.is_revealed {
                        if cell.is_mine {
                            "*"
                        } else {
                            cell.adjacent_mines.to_string().to_string().as_str()
                        }
                    } else {
                        if cell.is_flagged {
                            "F"
                        } else {
                            "#"
                        }
                    }
                )
            } else {
                format!(
                    " {} ",
                    if cell.is_revealed {
                        if cell.is_mine {
                            "*"
                        } else {
                            cell.adjacent_mines.to_string().as_str()
                        }
                    } else {
                        if cell.is_flagged {
                            "F"
                        } else {
                            "#"
                        }
                    }
                )
            };

            let cell_paragraph = Paragraph::new(cell_display)
                .block(Block::default().borders(Borders::ALL))
                .alignment(Alignment::Center);
            frame.render_widget(cell_paragraph, cell_layout[0]);
        }
    }
}

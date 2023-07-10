use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io, str::Chars};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

struct Theme {
    active_row_input_color: Color,
    border_color: Color,
    header_text_error_color: Color,
    header_text_success_color: Color,
    empty_row_block_color: Color,
    guess_in_right_place_color: Color,
    guess_in_word_color: Color,
    guess_not_in_word_color: Color,
    keyboard_not_guessed_color: Color,
    keyboard_in_right_place_color: Color,
    keyboard_in_word_color: Color,
    keyboard_not_in_word_color: Color,
    row_border_thickness: BorderType,
    guessed_row_border_thickness: BorderType,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark_theme()
    }
}

impl Theme {
    pub fn light_theme() -> Self {
        Self {
            border_color: Color::Black,
            active_row_input_color: Color::Black,
            header_text_success_color: Color::Green,
            header_text_error_color: Color::Red,
            empty_row_block_color: Color::Gray,
            guess_in_right_place_color: Color::Green,
            guess_in_word_color: Color::Yellow,
            guess_not_in_word_color: Color::DarkGray,
            keyboard_not_guessed_color: Color::Black,
            keyboard_in_right_place_color: Color::Green,
            keyboard_in_word_color: Color::Yellow,
            keyboard_not_in_word_color: Color::Gray,
            row_border_thickness: BorderType::Plain,
            guessed_row_border_thickness: BorderType::Thick,
        }
    }

    pub fn dark_theme() -> Self {
        Theme {
            border_color: Color::White,
            active_row_input_color: Color::White,
            keyboard_not_guessed_color: Color::White,
            keyboard_not_in_word_color: Color::Gray,
            ..Theme::light_theme()
        }
    }
}

pub struct BlockTheme {
    pub border_brightness: Modifier,
    pub border_color: Color,
    pub border_thickness: BorderType,
    pub text_color: Color,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum CharacterState {
    WrongPlace,
    Correct,
    NotInWord,
    Unknown,
    Masked
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum RowState {
    Empty,
    Current,
    AlreadyGuessed,
}

#[derive(Clone, Debug, PartialEq)]
enum GameState {
    InProgress,
    Won,
    Lost(String),
}

#[derive(Clone, Debug, PartialEq)]
struct Row {
    guess: String,
    char_states: [CharacterState; 5],
}

impl Row {
    fn from_current(app: &mut App) -> Row {
        let mut char_states = [CharacterState::Unknown; 5];
        for (char_idx, (correct_char, char)) in app
            .correct_word
            .clone()
            .chars()
            .zip(app.input.clone().chars())
            .enumerate()
        {
                            if !app.mask.get_mask(app.current_guess, char_idx)  {
            if correct_char == char {
                char_states[char_idx] = CharacterState::Correct;
            }
                }
        }

        for (char_idx, char) in app.input.clone().chars().enumerate() {
                            if !app.mask.get_mask(app.current_guess, char_idx)  {
            if char_states[char_idx] == CharacterState::Unknown {
                if app.correct_word.contains(char) {
                    char_states[char_idx] = CharacterState::WrongPlace;
                } else {
                    char_states[char_idx] = CharacterState::NotInWord;
                }
            }
                }
        }
        let guess = app.input.drain(..).collect::<String>();
        Row {
            guess,
            char_states: char_states.try_into().unwrap(), // Shouldn't fail since input is always 5 characters long
                                                          // state: RowState::AlreadyGuessed,
        }
    }

    fn new(mask: Mask, row_idx: usize) -> Self {
        let char_states = (0..5).map(|x| {
            let masked = mask.get_mask(row_idx, x);
            if masked {
                CharacterState::Masked
            } else {
                CharacterState::Unknown
                
            }
        }).collect::<Vec<CharacterState>>().try_into().unwrap();
        Self {
            guess: " ".to_string(),
            char_states,
        }
    }

    fn chars(&self) -> Chars<'_> {
        self.guess.chars()
    }
}

impl Default for Row {
    fn default() -> Self {
        Row {
            guess: "".to_string(),
            char_states: [CharacterState::Unknown; 5],
        }
    }
}

struct Mask {
    items: [bool; 25],
}

impl Mask {
    fn get_mask(&self, row_idx: usize, char_idx: usize) -> bool {
        self.items[(row_idx * 5) + char_idx]
    }    
}

impl Default for Mask {
    #[rustfmt::skip]
    fn default() -> Self {
        Self {
            items: [
             false, false, true, false, false,   
             false, true, false, false, false,   
             false, true, false, false, false,   
             false, false, true, false, false,   
             false, false, true, false, false,   
            ]
        }
    }
}


/// App holds the state of the application
struct App {
    input: String,
    guesses: Vec<Row>,
    correct_word: String,
    current_guess: usize,
    key_status: [CharacterState; 26],
    theme: Theme,
    state: GameState,
    mask: Mask,
}

impl App {
    fn get_letter_state(&self, c: char) -> CharacterState {
        match c {
            'a' => self.key_status[0],
            'b' => self.key_status[1],
            'c' => self.key_status[2],
            'd' => self.key_status[3],
            'e' => self.key_status[4],
            'f' => self.key_status[5],
            'g' => self.key_status[6],
            'h' => self.key_status[7],
            'i' => self.key_status[8],
            'j' => self.key_status[9],
            'k' => self.key_status[10],
            'l' => self.key_status[11],
            'm' => self.key_status[12],
            'n' => self.key_status[13],
            'o' => self.key_status[14],
            'p' => self.key_status[15],
            'q' => self.key_status[16],
            'r' => self.key_status[17],
            's' => self.key_status[18],
            't' => self.key_status[19],
            'u' => self.key_status[20],
            'v' => self.key_status[21],
            'w' => self.key_status[22],
            'x' => self.key_status[23],
            'y' => self.key_status[24],
            'z' => self.key_status[25],
            _ => CharacterState::Unknown,
        }
    }

    fn set_letter_state(&mut self, c: char, state: CharacterState) {
        match c {
            'a' => self.key_status[0] = state,
            'b' => self.key_status[1] = state,
            'c' => self.key_status[2] = state,
            'd' => self.key_status[3] = state,
            'e' => self.key_status[4] = state,
            'f' => self.key_status[5] = state,
            'g' => self.key_status[6] = state,
            'h' => self.key_status[7] = state,
            'i' => self.key_status[8] = state,
            'j' => self.key_status[9] = state,
            'k' => self.key_status[10] = state,
            'l' => self.key_status[11] = state,
            'm' => self.key_status[12] = state,
            'n' => self.key_status[13] = state,
            'o' => self.key_status[14] = state,
            'p' => self.key_status[15] = state,
            'q' => self.key_status[16] = state,
            'r' => self.key_status[17] = state,
            's' => self.key_status[18] = state,
            't' => self.key_status[19] = state,
            'u' => self.key_status[20] = state,
            'v' => self.key_status[21] = state,
            'w' => self.key_status[22] = state,
            'x' => self.key_status[23] = state,
            'y' => self.key_status[24] = state,
            'z' => self.key_status[25] = state,
            _ => (),
        }
    }
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            guesses: vec![
                Row::new(Mask::default(), 0 ),
                Row::new(Mask::default(), 1 ),
                Row::new(Mask::default(), 2 ),
                Row::new(Mask::default(), 3 ),
                Row::new(Mask::default(), 4 ),
            ],
            current_guess: 0,
            correct_word: "world".to_string().to_ascii_lowercase(),
            key_status: [CharacterState::Unknown; 26],
            theme: Theme::dark_theme(),
            state: GameState::InProgress,
            mask: Mask::default(),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::default();
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Enter => {
                    if valid_guess(app.input.clone()) {
                        // This is purely for the keyboard
                        for (char_idx, (correct_char, char)) in app
                            .correct_word
                            .clone()
                            .chars()
                            .zip(app.input.clone().chars()).enumerate()
                        {
                            if !app.mask.get_mask(app.current_guess, char_idx)  {
                                if correct_char == char {
                                    app.set_letter_state(char, CharacterState::Correct);
                                }
                            }
                        }

                        for (char_idx, char) in app.input.clone().chars().enumerate() {
                            if !app.mask.get_mask(app.current_guess, char_idx)  {
                                if app.get_letter_state(char) == CharacterState::Unknown {
                                    if app.correct_word.contains(char) {
                                        app.set_letter_state(char, CharacterState::WrongPlace);
                                    } else {
                                        app.set_letter_state(char, CharacterState::NotInWord);
                                    }
                                }
                            } 
                        }

                        if app.correct_word == app.input.as_str() {
                            app.state = GameState::Won;
                        }

                        app.guesses[app.current_guess] = Row::from_current(&mut app);

                        app.current_guess += 1;

                        if app.current_guess == 5 && app.state != GameState::Won {
                            app.state = GameState::Lost(app.correct_word.clone());
                        }
                    }
                }
                KeyCode::Char(c) => {
                    if app.state == GameState::InProgress {
                        if c != ' ' && app.input.len() < 5 && app.current_guess < 5 {
                            app.input.push(c.to_ascii_lowercase());
                        }
                    } else {
                        if c == 'q' {
                            return Ok(());
                        }
                    }
                }
                KeyCode::Backspace => {
                    app.input.pop();
                }
                KeyCode::Esc => {
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}

fn valid_guess(s: String) -> bool {
    if s.len() == 5 {
        true
    } else {
        false
    }
}

const ROWS: usize = 6;
const COLUMNS: usize = 5;
const CELL_WIDTH: usize = 5;
const CELL_HEIGHT: usize = 3;
const PADDING: usize = 1;

fn ui<B: Backend>(frame: &mut Frame<B>, app: &mut App) {
    let terminal_rect = frame.size();
    let grid_width = (CELL_WIDTH * COLUMNS + 2 * PADDING) as u16;
    let grid_height = (CELL_HEIGHT * ROWS + 2 * PADDING) as u16;

    let row_constraints = std::iter::repeat(Constraint::Length((CELL_HEIGHT) as u16))
        .take(ROWS)
        .collect::<Vec<_>>();

    let col_constraints = std::iter::repeat(Constraint::Length(CELL_WIDTH as u16))
        .take(COLUMNS)
        .collect::<Vec<_>>();

    let outer_rects = Layout::default()
        .direction(Direction::Vertical)
        .vertical_margin(1)
        .horizontal_margin(1)
        .constraints(vec![Constraint::Min(grid_height)])
        .split(frame.size());

    let game_rectangle = outer_rects[0];

    let horizontal_pad_block_width = (terminal_rect.width - grid_width) / 2;
    let center_center_horizontally = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Min(horizontal_pad_block_width),
            Constraint::Length(grid_width),
            Constraint::Min(horizontal_pad_block_width),
        ])
        .split(game_rectangle);

    let vertical_pad_block_height = (game_rectangle.height - grid_height) / 2;
    let center_content_vertically = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Min(vertical_pad_block_height),
            Constraint::Length(grid_height),
            Constraint::Min(vertical_pad_block_height),
        ])
        .split(center_center_horizontally[1]);

    let top_section_render_thing = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(center_content_vertically[0]);

    let keyboard_render_things = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(center_content_vertically[2]);

    let game_board = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let game_board_section = center_content_vertically[1];
    frame.render_widget(game_board, game_board_section);
    draw_header(frame, app, top_section_render_thing[0]);
    draw_keyboard(frame, app, keyboard_render_things[1]);

    let row_chunks = Layout::default()
        .direction(Direction::Vertical)
        .vertical_margin(1)
        .horizontal_margin(0)
        .constraints(row_constraints.clone())
        .split(game_board_section);


    for (row_index, _) in app.guesses.clone().iter().enumerate() {
        let row = row_chunks[row_index];

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .vertical_margin(0)
            .horizontal_margin(1)
            .constraints(col_constraints.clone())
            .split(row);

        let row_state = if row_index == app.current_guess {
            RowState::Current
        } else if row_index > app.current_guess {
            RowState::Empty
        } else {
            RowState::AlreadyGuessed
        };

        match row_state {
            RowState::Current => render_active_row(frame, app, chunks),
            RowState::Empty => render_empty_row(frame, app, chunks, row_index),
            RowState::AlreadyGuessed => render_already_guessed_row(frame, app, row_index, chunks),
        }
    }
}

fn render_empty_row<B: Backend>(frame: &mut Frame<B>, app: &mut App, cell_chunks: Vec<Rect>, row_index: usize) -> () {
    for (idx, cell_chunk) in cell_chunks.into_iter().enumerate() {
        let brightness = if app.guesses[row_index].char_states[idx] == CharacterState::Masked {
            Modifier::REVERSED
        } else {
            Modifier::empty()
        };
        let content = render_cell_with_text_and_colors(
            ' ',
            BlockTheme {
                border_color: app.theme.empty_row_block_color,
                text_color: app.theme.empty_row_block_color,
                border_thickness: app.theme.row_border_thickness,
                border_brightness: brightness,
            },
        );

        frame.render_widget(content, cell_chunk);
    }
}

fn render_active_row<B: Backend>(
    frame: &mut Frame<B>,
    app: &mut App,
    cell_chunks: Vec<Rect>,
) -> () {
    let mut chars = app.input.chars();

    for (idx, cell_chunk) in cell_chunks.into_iter().enumerate() {
        let text = match chars.next() {
            Some(l) => l,
            _ => ' ',
        };
        let brightness = if app.guesses[app.current_guess].char_states[idx] == CharacterState::Masked {
            Modifier::REVERSED
        } else {
            Modifier::empty()
        };
        let content = render_cell_with_text_and_colors(
            text,
            BlockTheme {
                border_color: app.theme.border_color,
                text_color: app.theme.active_row_input_color,
                border_thickness: app.theme.row_border_thickness,
                border_brightness: brightness,
            },
        );
        frame.render_widget(content, cell_chunk);
    }
}

fn render_already_guessed_row<B: Backend>(
    frame: &mut Frame<B>,
    app: &mut App,
    row_index: usize,
    chunks: Vec<Rect>,
) -> () {
    if let Some(word_guess) = app.guesses.get(row_index) {
        let items = chunks.iter().zip(word_guess.chars());

        for (char_id, (chunk, character)) in items.enumerate() {
            let accuracy = app.guesses[row_index].char_states[char_id];

            let color = match accuracy {
                CharacterState::Correct => app.theme.guess_in_right_place_color,
                CharacterState::WrongPlace => app.theme.guess_in_word_color,
                CharacterState::NotInWord => app.theme.guess_not_in_word_color,
                CharacterState::Unknown => app.theme.keyboard_not_guessed_color,
                CharacterState::Masked => app.theme.active_row_input_color
            };

            let brightness = match accuracy {
                CharacterState::WrongPlace => Modifier::DIM,
                CharacterState::Masked => Modifier::REVERSED,
                _ => Modifier::empty(),
            };

            let content = render_cell_with_text_and_colors(
                character,
                BlockTheme {
                    border_color: color,
                    text_color: color,
                    border_thickness: app.theme.guessed_row_border_thickness,
                    border_brightness: brightness,
                },
            );

            frame.render_widget(content, *chunk);
        }
    }
}

fn render_cell_with_text_and_colors(text: char, block_theme: BlockTheme) -> Paragraph<'static> {
    let text = formatted_cell_text(text);

    Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(block_theme.border_thickness)
                .border_style(Style::default().fg(block_theme.border_color))
                .style(
                    Style::default()
                        .add_modifier(block_theme.border_brightness)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .alignment(Alignment::Center)
        .style(Style::default().fg(block_theme.text_color))
}

// This is taken directly from the minesweeper app
// https://github.com/cpcloud/minesweep-rs/blob/main/src/ui.rs
fn formatted_cell_text(text: char) -> String {
    let single_row_text = format!("{:^length$}", text, length = CELL_WIDTH - 2);
    let pad_line = " ".repeat(CELL_WIDTH);
    let num_pad_lines = CELL_HEIGHT - 3;

    std::iter::repeat(pad_line.clone())
        .take(num_pad_lines / 2)
        .chain(std::iter::once(single_row_text.clone()))
        .chain(std::iter::repeat(pad_line).take(num_pad_lines / 2))
        .collect::<Vec<_>>()
        .join("\n")
}

fn draw_header<B: Backend>(frame: &mut Frame<B>, app: &mut App, chunk: Rect) {
    let text = match &app.state {
        GameState::Won => String::from("Game is over! You win! Press q or esc key to exit."),
        GameState::Lost(answer) => {
            format!("Game over! The answer was '{answer}'. Press q or esc key to exit.")
        }
        GameState::InProgress => String::from(""),
    };

    let header_text_color = match &app.state {
        GameState::Won => app.theme.header_text_success_color,
        _ => app.theme.header_text_error_color,
    };

    let header_text = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(header_text_color))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(app.theme.border_color))
                .title("Spotle Tui")
                .border_type(BorderType::Plain),
        );

    frame.render_widget(header_text, chunk);
}

fn draw_keyboard<B: Backend>(frame: &mut Frame<B>, app: &mut App, chunk: Rect) {
    let keyboard_key_rows = vec!["qwertyuiop", "asdfghjkl", "zxcvbnm"];
    let keyboard_spans = keyboard_key_rows
        .iter()
        .fold(vec![], |mut acc, keyboard_row| {
            // when we draw the keyboard, we want a blank space after every character
            // except for the last character, so that we don't go off-center
            let letters: Vec<Span> = keyboard_row
                .chars()
                .into_iter()
                .enumerate()
                .map(|(letter_index, letter)| {
                    let use_offset = letter_index != keyboard_row.len() - 1;
                    keyboard_letter(&app, letter, use_offset)
                })
                .collect();

            acc.push(Spans::from(letters));
            acc
        });

    let keyboard_visualization = Paragraph::new(keyboard_spans)
        .style(Style::default())
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(app.theme.border_color))
                .title("Available Letters")
                .border_type(BorderType::Plain),
        );

    frame.render_widget(keyboard_visualization, chunk);
}

fn keyboard_letter<'a>(app: &'a App, le: char, use_offset: bool) -> Span<'a> {
    use CharacterState::*;
    let key_state = app.get_letter_state(le);

    let color = match key_state {
        Unknown => app.theme.keyboard_not_guessed_color,
        Correct => app.theme.keyboard_in_right_place_color,
        WrongPlace => app.theme.keyboard_in_word_color,
        NotInWord => app.theme.keyboard_not_in_word_color,
        Masked => app.theme.active_row_input_color
    };

    let display_modifier = match key_state {
        NotInWord => Modifier::DIM,
        _ => Modifier::empty(),
    };

    let key_string = match use_offset {
        true => format!("{le} "),
        false => le.to_string(),
    };

    Span::styled(
        key_string,
        Style::default().fg(color).add_modifier(display_modifier),
    )
}

use crossterm::{
    ExecutableCommand,
    event::{self, KeyCode::*, KeyEvent, KeyEventKind},
    terminal::{self, ClearType},
};
use rand::random_range;
use tetris_core_rs::{
    AttackedLine, Cell, DEFAULT_BOARD_SIZE, MovementCommand, TetrisConfig, TetrisManager,
};
fn cast_to_readable(f: &Vec<Vec<Cell>>) -> String {
    let mut a = String::new();
    a += "\n";
    for y in f.iter() {
        for x in y.iter() {
            match x {
                &Cell::MinoInMotion(_) => a += "◆",
                &Cell::Empty => a += "□",
                &Cell::Wall => a += "■",
                &Cell::Obstruction(_) => a += "■",
                &Cell::MinoBlock(_) => a += "■",
                &Cell::Ghost(_) => a += "◇",
            };
        }

        a += "\n"
    }
    a
}
fn cast_to_readable_with_preview(f: &Vec<Vec<(bool, Cell)>>) -> String {
    let mut a = String::new();
    a += "\n";
    for y in f.iter() {
        for x in y.iter() {
            if x.0 {
                a += "×";
                continue;
            }
            match x.1 {
                Cell::MinoInMotion(_) => a += "◆",
                Cell::Empty => a += "□",
                Cell::Wall => a += "■",
                Cell::Obstruction(_) => a += "■",
                Cell::MinoBlock(_) => a += "■",
                Cell::Ghost(_) => a += "◇",
            };
        }

        a += "\n"
    }
    a
}
fn main() {
    let mut tetris_manager = TetrisManager::new(
        TetrisConfig::default(),
        &0,
        DEFAULT_BOARD_SIZE.0,
        DEFAULT_BOARD_SIZE.1,
    );
    use std::io::{self, Write};
    let mut stdout = io::stdout();
    stdout.execute(terminal::Clear(ClearType::All)).unwrap();
    stdout.flush().unwrap();
    write!(stdout, "press any key..").unwrap();
    /*write!(
        stdout,
        "{}",
        Cell::human_can(&tetris_manager.get_field_to_draw())
    );*/
    loop {
        if event::poll(std::time::Duration::from_millis(100)).unwrap() {
            if let event::Event::Key(KeyEvent {
                code,
                kind: KeyEventKind::Press,
                ..
            }) = event::read().unwrap()
            {
                let command = match code {
                    Char('q') => {
                        break; // 'q' を押すと終了
                    }
                    Char('z') => MovementCommand::RotateCounterClockWise,
                    Char('x') => MovementCommand::RotateClockWise,
                    Char('c') => MovementCommand::Hold,
                    Left => MovementCommand::Left,
                    Right => MovementCommand::Right,
                    Down => MovementCommand::Down,
                    Char(' ') => MovementCommand::HardDrop,
                    Char('r') => {
                        tetris_manager.command(MovementCommand::Attacked(AttackedLine {
                            hole_indexes: Some([random_range(0..DEFAULT_BOARD_SIZE.0)].to_vec()),
                            can_be_cleared: true,
                        }));
                        continue;
                    }
                    _ => {
                        continue;
                    }
                };
                let r = tetris_manager.command(command);
                if r.0.is_err() {
                    panic!("lose");
                } else {
                    write!(stdout, "{:?}", r).unwrap();
                }

                write!(stdout, "{:?}\n", tetris_manager.get_next_minos(7)).unwrap();
                if let Some(mino_type) = tetris_manager.get_hold_mino() {
                    write!(stdout, "{:?}\n", mino_type).unwrap();
                } else {
                    write!(stdout, "None\n").unwrap();
                }
                write!(
                    stdout,
                    "{}",
                    serde_json::to_string(&tetris_manager).unwrap()
                )
                .unwrap();

                write!(
                    stdout,
                    "{}",
                    cast_to_readable_with_preview(
                        &tetris_manager.get_field_to_draw_with_preview_next_mino()
                    )
                )
                .unwrap();
                stdout.flush().unwrap();
            }
        }
        stdout.flush().unwrap(); // 即座に出力を反映
    }
}

use actix_web::{error, web};
use anyhow::bail;
use std::{fmt::Display, str::FromStr, sync::Mutex};
use tracing::instrument;

#[derive(Debug, Default)]
pub struct Board {
    /// (0,0) is bottom left columns[0][3] is top left
    columns: [[CellValue; 4]; 4],
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
enum CellValue {
    #[default]
    Empty,
    Cookie,
    Milk,
}

impl From<Team> for CellValue {
    fn from(value: Team) -> Self {
        match value {
            Team::Cookie => Self::Cookie,
            Team::Milk => Self::Milk,
        }
    }
}
impl CellValue {
    fn as_str(&self) -> &'static str {
        match self {
            CellValue::Empty => "⬛",
            CellValue::Cookie => "🍪",
            CellValue::Milk => "🥛",
        }
    }

    fn is_occupied(self) -> bool {
        !self.is_empty()
    }

    fn is_empty(self) -> bool {
        self == Self::Empty
    }
}

impl TryFrom<CellValue> for Team {
    type Error = anyhow::Error;

    fn try_from(value: CellValue) -> Result<Self, Self::Error> {
        Ok(match value {
            CellValue::Empty => bail!("Empty is not a valid Team"),
            CellValue::Cookie => Team::Cookie,
            CellValue::Milk => Team::Milk,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Team {
    Cookie,
    Milk,
}

impl FromStr for Team {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cookie" => Ok(Self::Cookie),
            "milk" => Ok(Self::Milk),
            other => bail!("{other:?} is not a valid Team"),
        }
    }
}

impl Display for Team {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Team::Cookie => CellValue::Cookie,
                Team::Milk => CellValue::Milk,
            }
            .as_str()
        )
    }
}

impl Board {
    const WALL: &str = "⬜";
    fn new() -> Self {
        Self::default()
    }

    fn new_wrapped() -> web::Data<Mutex<Self>> {
        web::Data::new(Mutex::new(Self::new()))
    }
    fn row_as_str(&self, row: usize) -> String {
        let mut result = String::new();
        for col in 0..4 {
            result.push_str(self.columns[col][row].as_str());
        }
        result
    }

    fn reset(&mut self) {
        self.columns = Default::default();
    }

    fn winner_status(&self) -> Option<String> {
        let winner = self.winner()?;
        if let Some(team) = winner {
            Some(format!("{team} wins!"))
        } else {
            Some("No winner.".into())
        }
    }

    /// Outer option represent if there is any winner and inner option represent which team one if any
    fn winner(&self) -> Option<Option<Team>> {
        // Check diagonals
        if self.columns[3][3].is_occupied()
            && (0..3).all(|i| self.columns[i][i] == self.columns[i + 1][i + 1])
        {
            return Some(Some(
                self.columns[0][0]
                    .try_into()
                    .expect("checked that it is not empty in if condition"),
            ));
        }
        if self.columns[0][3].is_occupied()
            && (0..3).all(|i| self.columns[i][3 - i] == self.columns[i + 1][3 - i - 1])
        {
            return Some(Some(
                self.columns[0][3]
                    .try_into()
                    .expect("checked that it is not empty in if condition"),
            ));
        }

        // Check each row and column
        for i in 0..4 {
            // Check row
            if self.columns[0][i].is_occupied()
                && (0..3).all(|col| self.columns[col][i] == self.columns[col + 1][i])
            {
                return Some(Some(
                    self.columns[0][i]
                        .try_into()
                        .expect("check that is it not empty in if condition"),
                ));
            }

            // Check column
            if self.columns[i][3].is_occupied()
                && (0..3).all(|row| self.columns[i][row] == self.columns[i][row + 1])
            {
                return Some(Some(
                    self.columns[i][3]
                        .try_into()
                        .expect("check that is it not empty in if condition"),
                ));
            }
        }

        // Check for game over but no winner (game over if no spaces in top row)
        if (0..4).all(|col| self.columns[col][3].is_occupied()) {
            return Some(None);
        }

        // No matches game is not over
        None
    }

    fn place(&mut self, team: Team, col: usize) -> actix_web::Result<()> {
        if self.winner().is_some() {
            return Err(error::ErrorServiceUnavailable(self.to_string()));
        }

        for i in 0..4 {
            if self.columns[col][i].is_empty() {
                self.columns[col][i] = team.into();
                return Ok(());
            }
        }

        Err(error::ErrorServiceUnavailable("column full"))
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in (0..4).rev() {
            writeln!(f, "{}{}{}", Self::WALL, self.row_as_str(row), Self::WALL,)?;
        }
        writeln!(f, "{}", Self::WALL.repeat(6))?;
        if let Some(game_ended_status) = self.winner_status() {
            writeln!(f, "{game_ended_status}")
        } else {
            Ok(())
        }
    }
}

#[instrument]
async fn show_board(board: web::Data<Mutex<Board>>) -> String {
    board.lock().unwrap().to_string()
}

#[instrument]
async fn reset(board: web::Data<Mutex<Board>>) -> String {
    let mut guard = board.lock().unwrap();
    guard.reset();
    guard.to_string()
}

#[instrument]
async fn place(
    path: web::Path<(String, String)>,
    board: web::Data<Mutex<Board>>,
) -> actix_web::Result<String> {
    let (team, col) = path.into_inner();
    let team: Team = team.parse().map_err(error::ErrorBadRequest)?;
    let col: usize = col.parse().map_err(error::ErrorBadRequest)?;
    let mut guard = board.lock().unwrap();
    if !(1..=4).contains(&col) {
        return Err(error::ErrorBadRequest(""));
    }
    guard.place(team, col - 1)?;
    Ok(guard.to_string())
}

pub fn scope() -> actix_web::Scope {
    web::scope("/12")
        .route("/board", web::get().to(show_board))
        .route("/reset", web::post().to(reset))
        .route("/place/{team}/{col}", web::post().to(place))
}

pub fn app_data() -> web::Data<Mutex<Board>> {
    Board::new_wrapped()
}

use std::sync::Arc;

use axum::{extract::{Path, State}, http::StatusCode, routing::{get, post}, Router};
use itertools::Itertools;
use rand::{rngs::StdRng, Rng, SeedableRng};
use serde::Deserialize;
use tokio::sync::RwLock;


/// |0,3|1,3|2,3|3,3|
/// |0,2|1,2|2,2|3,2|
/// |0,1|1,1|2,1|3,1|
/// |0,0|1,0|2,0|3,0|
/// +---+---+---+---+


#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Item {
    Cookie,
    Milk,
}

#[derive(Debug, Default)]
enum GameState {
    #[default]
    Normal,
    Wins(Item),
    NoWinner,
}

#[derive(Debug)]
struct AppState {
    slots: [Vec<Item>; 4],
    game_state: GameState,
    rng: StdRng,
}

#[derive(Debug, Deserialize)]
struct PlaceParam {
    item: Item,
    position: u16,
}

impl Item {
    const fn as_char(&self) -> char {
        match self {
            Self::Cookie => '🍪',
            Self::Milk => '🥛',
        }
    }
}

impl GameState {
    const fn to_str(&self) -> &'static str {
        match self {
            Self::Normal => "",
            Self::Wins(Item::Cookie) => "🍪 wins!\n",
            Self::Wins(Item::Milk) => "🥛 wins!\n",
            Self::NoWinner => "No winner.\n"
        }
    }
}

impl AppState {

    const WINNING_COORDS_NT: [[(usize, usize); 4]; 10] = [
        [(0, 0), (0, 1), (0, 2), (0, 3)],
        [(1, 0), (1, 1), (1, 2), (1, 3)],
        [(2, 0), (2, 1), (2, 2), (2, 3)],
        [(3, 0), (3, 1), (3, 2), (3, 3)],

        [(0, 0), (1, 0), (2, 0), (3, 0)],
        [(0, 1), (1, 1), (2, 1), (3, 1)],
        [(0, 2), (1, 2), (2, 2), (3, 2)],
        [(0, 3), (1, 3), (2, 3), (3, 3)],

        [(0, 0), (1, 1), (2, 2), (3, 3)],
        [(0, 3), (1, 2), (2, 1), (3, 0)],
    ];

    pub fn new() -> Self {
        Self {
            slots: <_>::default(),
            game_state: <_>::default(),
            rng: StdRng::seed_from_u64(2024),
        }
    }

    pub fn get_display_string(&self) -> String {
        let mut board_string = String::new();

        for i in (0..4).rev() {
            board_string.push('⬜');
            for j in 0..4 {
                let item = self.slots[j].get(i).map(Item::as_char);
                board_string.push(item.unwrap_or('⬛'));
            }
            board_string.push('⬜');
            board_string.push('\n');
        }
        board_string.push_str("⬜⬜⬜⬜⬜⬜\n");
        board_string.push_str(self.game_state.to_str());

        board_string
    }

    pub fn reset(&mut self) {
        for slot in self.slots.iter_mut() {
            slot.clear();
        }
        self.game_state = GameState::Normal;
    }

    #[inline]
    pub fn reset_rng(&mut self) {
        self.rng = StdRng::seed_from_u64(2024);
    }

    pub fn place(&mut self, item: Item, position: u16) -> bool {
        if position >= 4 || !matches!(self.game_state, GameState::Normal) {
            return false;
        }
        let slot = &mut self.slots[position as usize];
        if slot.len() >= 4 {
            return false;
        }
        slot.push(item);
        self.update_game_state();

        true
    }

    fn update_game_state(&mut self) {
        for coords in Self::WINNING_COORDS_NT {
            if let Ok(Some(item)) = coords
                .iter()
                .map(|(i, j)| self.slots[*i].get(*j))
                .all_equal_value()
            {
                self.game_state = GameState::Wins(*item);
                return;
            }
        }

        if self.slots.iter().all(|slot| slot.len() >= 4) {
            self.game_state = GameState::NoWinner;
            return;
        }

        self.game_state = GameState::Normal;
    }

    pub fn random_board(&mut self) {
        self.reset();
        for _ in 0..4 {
            for i in 0..4 {
                self.slots[i].insert(0, if self.rng.gen::<bool>() { Item::Cookie } else { Item::Milk });
            }
        }
        self.update_game_state();
    }
}

async fn board(State(state): State<Arc<RwLock<AppState>>>) -> String {
    state.read().await.get_display_string()
}

async fn reset(State(state): State<Arc<RwLock<AppState>>>) -> String {
    let mut state = state.write().await;
    state.reset();
    state.reset_rng();
    let state = state.downgrade();
    state.get_display_string()
}

async fn place(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(PlaceParam { item, position }): Path<PlaceParam>,
) -> Result<(StatusCode, String), StatusCode> {
    let mut state = state.write().await;
    if !(1..=4).contains(&position) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let place_succeed = state.place(item, position - 1);
    let state = state.downgrade();
    Ok((if place_succeed { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE }, state.get_display_string()))
}

async fn random_board(State(state): State<Arc<RwLock<AppState>>>) -> String {
    let mut state = state.write().await;
    state.random_board();
    let state = state.downgrade();
    state.get_display_string()
}

#[inline(always)]
pub fn router() -> Router {
    Router::new()
        .route("/board", get(board))
        .route("/reset", post(reset))
        .route("/place/:item/:position", post(place))
        .route("/random-board", get(random_board))
        .with_state(Arc::new(RwLock::new(AppState::new())))
}

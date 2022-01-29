use std::{collections::{HashMap, HashSet}, iter::once};
use crate::util::{SCResult, FromXmlNode, XmlNode};
use super::{BOARD_SIZE, Board, CORNERS, Color, Move, PIECE_SHAPES, PIECE_SHAPES_BY_NAME, Piece, PieceShape, Player, Team, Vec2};

/// A snapshot of the game's state. It holds the
/// information needed to compute the next move.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameState {
    /// The number of already committed moves.
    pub turn: u32,
    /// The number of rounds.
    pub round: u32,
    /// The first team's player.
    pub first: Player,
    /// The second team's player.
    pub second: Player,
    /// The current game board.
    pub board: Board,
    /// The piece that has to be placed in the first round.
    pub start_piece: PieceShape,
    /// The color that begins the game.
    pub start_color: Color,
    /// The team that begins the game.
    pub start_team: Team,
    /// A list of all colors currently in the game.
    pub ordered_colors: Vec<Color>,
    /// A map that stores, for each color, whether the last move was a monomino if all pieces have been placed.
    pub last_move_mono: HashMap<Color, bool>,
    /// The current color's index
    pub current_color_index: u32,
    /// The undeployed blue shapes.
    pub blue_shapes: HashSet<PieceShape>,
    /// The undeployed yellow shapes.
    pub yellow_shapes: HashSet<PieceShape>,
    /// The undeployed red shapes.
    pub red_shapes: HashSet<PieceShape>,
    /// The undeployed green shapes.
    pub green_shapes: HashSet<PieceShape>
}

const SUM_MAX_SQUARES: i32 = 89;

impl GameState {
    /// Creates a brand-new game state with blue as the starting color
    /// and team one as the starting team. Mostly for debugging purposes.
    pub fn new(start_piece: PieceShape) -> Self {
        GameState {
            turn: 0,
            round: 1,
            first: Player { team: Team::One, display_name: "Alice".to_owned() },
            second: Player { team: Team::Two, display_name: "Bob".to_owned() },
            board: Board::new(),
            start_piece,
            start_color: Color::Blue,
            start_team: Team::One,
            ordered_colors: vec![Color::Blue, Color::Yellow, Color::Red, Color::Green],
            last_move_mono: HashMap::new(),
            current_color_index: 0,
            blue_shapes: PIECE_SHAPES.iter().cloned().collect(),
            yellow_shapes: PIECE_SHAPES.iter().cloned().collect(),
            red_shapes: PIECE_SHAPES.iter().cloned().collect(),
            green_shapes: PIECE_SHAPES.iter().cloned().collect()
        }
    }

    /// Fetches the current color.
    pub fn current_color(&self) -> Color {
        self.ordered_colors[self.current_color_index as usize]
    }

    /// Fetches the current team.
    pub fn current_team(&self) -> Team {
        self.current_color().team()
    }

    /// Fetches the current player.
    pub fn current_player(&self) -> &Player {
        match self.current_team() {
            Team::One => &self.first,
            Team::Two => &self.second,
            Team::None => panic!("Cannot fetch the current player with the team being 'none'!")
        }
    }

    /// Fetches the undeployed piece shapes of a given color.
    pub fn undeployed_shapes_of_color(&self, color: Color) -> impl Iterator<Item=&PieceShape> {
        match color {
            Color::Red => self.red_shapes.iter(),
            Color::Yellow => self.yellow_shapes.iter(),
            Color::Green => self.green_shapes.iter(),
            Color::Blue => self.blue_shapes.iter(),
            Color::None => panic!("Cannot fetch shapes of color 'none'!")
        }
    }

    /// Fetches the undeployed piece shapes of a given color mutably.
    pub fn undeployed_shapes_of_color_mut(&mut self, color: Color) -> &mut HashSet<PieceShape> {
        match color {
            Color::Red => &mut self.red_shapes,
            Color::Yellow => &mut self.yellow_shapes,
            Color::Green => &mut self.green_shapes,
            Color::Blue => &mut self.blue_shapes,
            Color::None => panic!("Cannot fetch shapes of color 'none'!")
        }
    }

    // Game rule logic is mostly a direct translation of
    // https://github.com/software-challenge/backend/blob/97d185660754ffba4bd4444f3f39ae350f1d053e/plugin/src/shared/sc/plugin2021/util/GameRuleLogic.kt

    /// Computes the points from the given, undeployed piece shapes.
    pub fn get_points_from_undeployed(undeployed: HashSet<PieceShape>, mono_last: bool) -> i32 {
        // If all pieces were placed
        if undeployed.is_empty() {
            // Return sum of all squares plus 15 bonus points.
            // If the Monomino was the last placed piece, add another 5 points
            SUM_MAX_SQUARES + 15 + if mono_last { 5 } else { 0 }
        } else {
            // One point per piece placed
            let placed_points: i32 = undeployed.iter().map(|p| p.coordinates().count() as i32).sum();
            SUM_MAX_SQUARES - placed_points
        }
    }

    /// Whether the game state is in the first round.
    pub fn is_first_move(&self) -> bool {
        self.undeployed_shapes_of_color(self.current_color()).count() == PIECE_SHAPES.len()
    }

    /// Performs the given move.
    pub fn perform_move(&mut self, game_move: Move) -> SCResult<()> {
        #[cfg(debug_assertions)]
        self.validate_move_color(&game_move)?;

        match game_move {
            Move::Set { piece } => self.perform_set_move(piece),
            Move::Skip { .. } => self.perform_skip_move()
        }
    }

    /// Fetches the state after the given move.
    pub fn after_move(&self, game_move: Move) -> SCResult<GameState> {
        let mut s = self.clone();
        s.perform_move(game_move)?;
        Ok(s)
    }

    /// Checks whether the given move has the right color.
    fn validate_move_color(&self, game_move: &Move) -> SCResult<()> {
        if game_move.color() != self.current_color() {
            Err(format!("Move color {} does not match game state color {}!", game_move.color(), self.current_color()).into())
        } else {
            Ok(())
        }
    }

    /// Checks whether the given shape is valid.
    fn validate_shape(&self, shape: &PieceShape, color: Color) -> SCResult<()> {
        if self.is_first_move() {
            if shape != &self.start_piece {
                return Err(format!("{} is not the (requested) first shape", shape).into())
            }
        } else if !self.undeployed_shapes_of_color(color).any(|p| p == shape) {
            return Err(format!("Piece {} has already been placed before!", shape).into())
        }

        Ok(())
    }

    /// Checks whether the given set move is valid.
    fn validate_set_move(&self, piece: &Piece) -> SCResult<()> {
        self.validate_shape(&piece.kind, piece.color)?;

        for coordinates in piece.coordinates() {
            if !Board::is_in_bounds(coordinates) {
                return Err(format!("Target position of the set move {} is not in the board's bounds!", coordinates).into());
            }

            if self.board.is_obstructed(coordinates) {
                return Err(format!("Target position of the set move {} is obstructed!", coordinates).into());
            }

            if self.board.borders_on_color(coordinates, piece.color) {
                return Err(format!("Target position of the set move {} already borders on {}!", coordinates, piece.color).into());
            }
        }

        if self.is_first_move() {
            // Check whether it is placed correctly in a corner
            if !piece.coordinates().any(|p| Board::is_on_corner(p)) {
                return Err("The piece from the set move is not located in a corner!".into());
            }
        } else {
            // Check whether the piece is connected to at least one tile of the same color by corner
            if !piece.coordinates().any(|p| self.board.corners_on_color(p, piece.color)) {
                return Err(format!("The piece {:?} shares no corner with another piece of same color!", piece).into());
            }
        }

        Ok(())
    }

    pub fn try_advance(&mut self, turns: u32) -> SCResult<()> {
        if self.ordered_colors.is_empty() {
            return Err("Game has already ended, cannot advance!".into());
        }

        self.current_color_index = (self.current_color_index + turns) % self.ordered_colors.len() as u32;
        // TODO: This doesn't seem correct, but matches the implementation of https://github.com/CAU-Kiel-Tech-Inf/backend/blob/97d185660754ffba4bd4444f3f39ae350f1d053e/plugin/src/shared/sc/plugin2021/GameState.kt#L114-L123
        // Perhaps we should divide AFTER the turns have been added, then simply assign instead of add-assign the round?
        self.round += turns / self.ordered_colors.len() as u32;
        self.turn += turns;

        Ok(())
    }

    /// Performs the given set move.
    fn perform_set_move(&mut self, piece: Piece) -> SCResult<()> {
        #[cfg(debug_assertions)]
        self.validate_set_move(&piece)?;

        self.board.place(&piece);

        let undeployed = self.undeployed_shapes_of_color_mut(piece.color);
        undeployed.remove(&piece.shape());
        // TODO: Track deployed shapes
        
        // If this was the last piece for this color, remove it from the turn queue
        if undeployed.is_empty() {
            self.last_move_mono.insert(piece.color, piece.kind == PIECE_SHAPES_BY_NAME["MONO"]);
        }

        self.try_advance(1)?;
        Ok(())
    }

    /// Performs the given skip move
    fn perform_skip_move(&mut self) -> SCResult<()> {
        if self.is_first_move() {
            return Err("Cannot skip the first round!".into());
        }

        self.try_advance(1)?;
        Ok(())
    }

    fn validate_skip(&self) -> SCResult<()> {
        self.clone().try_advance(1)
    }

    /// Fetches the possible moves
    pub fn possible_moves(&self) -> impl Iterator<Item=Move> {
        if self.is_first_move() {
            self.possible_first_moves()
                .collect::<Vec<_>>()
                .into_iter()
        } else {
            self.possible_usual_set_moves()
                .chain(once(Move::Skip { color: self.current_color() }).filter(|_| self.validate_skip().is_ok()))
                .collect::<Vec<_>>()
                .into_iter()
        }
    }

    /// Fetches the possible non-start moves
    fn possible_usual_set_moves(&self) -> impl Iterator<Item=Move> {
        let color = self.current_color();
        self.undeployed_shapes_of_color(color)
            .flat_map(|kind| {
                let bb = kind.bounding_box();
                let placable = Vec2::both(BOARD_SIZE as i32 - 1) - bb;
                kind.transformations()
                    .flat_map(|(rotation, is_flipped)| placable
                        .into_iter()
                        .map(move |position| Piece {
                            kind: kind.clone(),
                            rotation,
                            is_flipped,
                            color,
                            position
                        })
                    )
                    .filter(|piece| self.validate_set_move(piece).is_ok())
                    .map(|piece| Move::Set { piece })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// Fetches the possible start moves
    fn possible_first_moves(&self) -> impl Iterator<Item=Move> {
        let kind = self.start_piece.clone();
        let color = self.current_color();
        kind
            .transformations()
            .flat_map(|(rotation, is_flipped)| {
                let k = kind.clone();
                CORNERS
                    .iter()
                    .map(move |&corner| Piece {
                        kind: k.clone(),
                        rotation,
                        is_flipped,
                        color,
                        position: Board::align(k.transform(rotation, is_flipped).bounding_box(), corner)
                    })
                    .filter(|piece| self.validate_set_move(piece).is_ok())
                    .map(|piece| Move::Set { piece })
            })
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl FromXmlNode for GameState {
    fn from_node(node: &XmlNode) -> SCResult<Self> {
        Ok(Self {
            turn: node.attribute("turn")?.parse()?,
            round: node.attribute("round")?.parse()?,
            first: Player::from_node(node.child_by_name("first")?)?,
            second: Player::from_node(node.child_by_name("second")?)?,
            board: Board::from_node(node.child_by_name("board")?)?,
            start_piece: node.attribute("startPiece")?.parse()?,
            start_color: Color::from_node(node.child_by_name("startColor")?)?,
            start_team: Team::from_node(node.child_by_name("startTeam")?)?,
            ordered_colors: node.child_by_name("orderedColors")?.childs_by_name("color").map(Color::from_node).collect::<Result<_, _>>()?,
            last_move_mono: HashMap::new(), // TODO
            current_color_index: node.attribute("currentColorIndex")?.parse()?,
            blue_shapes: node.child_by_name("blueShapes")?.childs_by_name("shape").map(PieceShape::from_node).collect::<Result<_, _>>()?,
            yellow_shapes: node.child_by_name("yellowShapes")?.childs_by_name("shape").map(PieceShape::from_node).collect::<Result<_, _>>()?,
            red_shapes: node.child_by_name("redShapes")?.childs_by_name("shape").map(PieceShape::from_node).collect::<Result<_, _>>()?,
            green_shapes: node.child_by_name("greenShapes")?.childs_by_name("shape").map(PieceShape::from_node).collect::<Result<_, _>>()?
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::game::{Color, Move, PIECE_SHAPES_BY_NAME, Team};

    use super::GameState;

    #[test]
    fn test_game_state() {
        let start_piece = "PENTO_Y";
        let mut state = GameState::new(PIECE_SHAPES_BY_NAME[start_piece].clone());

        // Verify that the initial setup is correct
        assert_eq!(state.current_color(), Color::Blue);
        assert_eq!(state.current_team(), Team::One);
        assert_eq!(state.start_color, state.current_color());
        assert_eq!(state.start_team, state.current_team());
        assert_eq!(state.ordered_colors[state.current_color_index as usize], state.current_color());
        assert_eq!(state.board.count_obstructed(), 0);
        assert!(state.is_first_move());

        {
            let possible_moves: Vec<_> = state.possible_moves().collect();
            let possible_first_moves: Vec<_> = state.possible_first_moves().collect();

            assert!(!possible_moves.is_empty());
            assert_eq!(possible_moves, possible_first_moves);
            
            let shapes = possible_moves.iter().cloned().map(|m|
                match m {
                    Move::Set { piece } => piece.shape().ascii_art().to_string(),
                    _ => panic!("Skip moves should never be first!")
                }
            ).map(|s| s.trim().to_string()).collect::<Vec<_>>();
            
            assert!(shapes.contains(&"#....\n\
                                      ##...\n\
                                      #....\n\
                                      #....\n\
                                      .....".to_string()));
            assert!(shapes.contains(&"####.\n\
                                      ..#..\n\
                                      .....\n\
                                      .....\n\
                                      .....".to_string()));
            assert!(shapes.contains(&"####.\n\
                                      .#...\n\
                                      .....\n\
                                      .....\n\
                                      .....".to_string()));
            assert!(shapes.contains(&"#....\n\
                                      #....\n\
                                      ##...\n\
                                      #....\n\
                                      .....".to_string()));
            
            state.perform_move(possible_moves[0].clone()).unwrap();
        }
        {
            let possible_moves: Vec<_> = state.possible_moves().collect();
            
            assert!(state.is_first_move());
            assert_eq!(state.current_color(), Color::Yellow);
            assert_eq!(state.current_team(), Team::Two);
            assert!(!possible_moves.is_empty());
        }
    }
}

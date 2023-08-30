use std::collections::HashMap;

type Uid = u32;
pub struct Game {
    pub uid: Uid,
    pub white_uid: Uid,
    pub black_uid: Uid,
}

#[derive(Debug)]
pub struct Player {
    pub uid: Uid,
    pub game: Option<Uid>,
}

pub struct MatchMaker {
    hanging_player: Option<Uid>,
    games: HashMap<Uid, Game>,
    players: HashMap<Uid, Player>,
}

impl MatchMaker {
    pub fn new() -> MatchMaker {
        MatchMaker {
            hanging_player: None,
            games: HashMap::new(),
            players: HashMap::new(),
        }
    }

    fn build_player(&self, uid: Uid) -> Player {
        Player {
            uid: uid,
            game: None,
        }
    }

    fn create_game(&mut self, uid_white: Uid, uid_black: Uid) -> Game {
        let game = Game {
            white_uid: uid_white,
            black_uid: uid_black,
            uid: 0,
        };
        let white_player_ref = self.players.get_mut(&game.white_uid).unwrap();
        white_player_ref.game = Some(game.uid);

        let black_player_ref = self.players.get_mut(&game.black_uid).unwrap();
        black_player_ref.game = Some(game.uid);

        println!(
            "New game created: {} between {} as white and {} as black",
            game.uid, game.white_uid, game.black_uid
        );
        return game;
    }

    // Tries to find a player to match with the given uid
    fn matchmake(&mut self, uid: Uid) {
        match self.hanging_player {
            None => {
                self.hanging_player = Some(uid);
            }
            Some(hanging_player) => {
                let game = self.create_game(hanging_player, uid);
                self.hanging_player = None;
                self.games.insert(game.uid, game);
            }
        }
    }

    pub fn on_new_player_connected(&mut self, uid: Uid) {
        println!("New player connected {}", uid);

        let new_player = self.build_player(uid);
        self.players.insert(uid, new_player);
        self.matchmake(uid);
    }

    pub fn on_player_says(&mut self, uid: Uid, message: String) {
        println!("Player {} says {}", uid, message);
    }

    pub fn on_player_disconnected(&mut self, uid: Uid) {
        println!("Player {} disconnected", uid);
        let player = self.players.get(&uid).unwrap();
        let game = player.game;

        // Was the player in a game?
        match game {
            None => {
                // It was the hanging player
                if self.hanging_player.unwrap() == uid {
                    self.hanging_player = None;
                } else {
                    panic!(
                        "Hanging player is not the one who disconnected, this should not happen"
                    );
                }
            }
            Some(game_uid) => {
                //The disconnected player was in a game
                let game = self.games.get_mut(&game_uid).unwrap();
                let winner_uid = if game.white_uid == uid {
                    game.black_uid
                } else {
                    game.white_uid
                };

                self.game_end(game_uid, winner_uid, uid);
                //removing game
                self.games.remove(&game_uid);
                //destroying the disconnected player
                self.players.remove(&uid);
                //setting the remaining as hanging player
                let winner_player = self.players.get_mut(&winner_uid).unwrap();
                winner_player.game = None;
                self.matchmake(winner_uid);
            }
        }

        self.destroy_player(uid);
    }

    fn destroy_player(&mut self, game_uid: Uid) {
        self.players.remove(&game_uid);
    }

    fn game_end(&mut self, game_uid: Uid, uid_win: Uid, uid_lose: Uid) {
        println!(
            "Game {} ended, {} won and {} lost.",
            game_uid, uid_win, uid_lose
        );
    }
}

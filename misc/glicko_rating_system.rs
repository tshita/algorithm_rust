//! Glicko Rating System
//! ==============================================================================
//!
//! # Description
//! 1995 年に Mark Glickman によって開発されたレーティングシステム。特徴などは以下の通り。
//! - Elo rating system は点推定しているが、Glicko rating system は RD (rating deviation) の信頼区間を加え区間推定に変更
//! - レーティングの 95% 信頼区間は約 (rating - 2RD, rating + 2RD) （c.f. get_95confidence_interval 関数）
//! - 自分と対戦相手とのレーティング増減量の絶対値は等しくない
//! - レーティングの更新は連続した複数の試合終了後に行う。この期間を Rating Period と呼び、例では 3 試合としている（期間は管理者が決任意に設定できるが、おすすめは 5-10 試合らしい）
//! - プレイヤーの試合数が多くなると RD が小さくなりレーティングの増減も小さくなる
//! - Glicko rating system を改良したものに Glicko-2 rating system がある
//!
//! # References
//! - [Mark E. Glickman: The Glicko system.](http://www.glicko.net/glicko/glicko.pdf)
//! - [Mark E. Glickman: Parameter estimation in large dynamic paired comparison experiments.](http://www.glicko.net/research/glicko.pdf)

use std::f64::consts::PI;

/// A player with a rating and a rating deviation.
#[derive(Clone)]
struct Player {
    rating: f64,
    rd: f64, // rating deviation (RD)
}

/// An outcome of a game consists of a loss, a win, and a draw.
#[derive(Debug)]
#[allow(unused)]
enum Outcome {
    Lose,
    Win,
    Draw,
}

/// An opponent and a their match result.
struct GameResult(Player, Outcome);

impl GameResult {
    // Only used in the update function
    fn to_f64(&self) -> f64 {
        match self.1 {
            Outcome::Win => 1.0,
            Outcome::Lose => 0.0,
            Outcome::Draw => 0.5,
        }
    }
}

impl Player {
    /// Generate an unrated player.
    #[allow(unused)]
    pub fn new() -> Self {
        Self {
            rating: 1500.0,
            rd: 350.0,
        }
    }

    /// Update the player's rating and RD.
    pub fn update(&mut self, game_results: &Vec<GameResult>) {
        // calculate 1 / d^2
        let dd_inv = Self::q().powf(2.0)
            * game_results
                .iter()
                .map(|o| o.0.g().powf(2.0) * o.0.e(self.rating) * (1.0 - o.0.e(self.rating)))
                .sum::<f64>();

        let sum = game_results
            .iter()
            .map(|o| o.0.g() * (o.to_f64() - o.0.e(self.rating)))
            .sum::<f64>();

        self.rating += Self::q() / (1.0 / self.rd.powf(2.0) + dd_inv) * sum;
        self.rd = (1.0 / (1.0 / (self.rd * self.rd) + dd_inv)).sqrt();
    }

    /// Calculate an expected outcome of a game with an opponent.
    pub fn expected_outcome(&self, opponent: &Player) -> f64 {
        let arg_g = Player {
            rating: 0.0,
            rd: (self.rd.powf(2.0) + opponent.rd.powf(2.0)).sqrt(),
        };
        let pow = -arg_g.g() * (self.rating - opponent.rating) / 400.0;
        1.0 / (1.0 + 10.0_f64.powf(pow))
    }

    /// Get a 95 % confidence interaval for the player.
    pub fn get_95confidence_interval(&self) -> (f64, f64) {
        (self.rating - 1.96 * self.rd, self.rating + 1.96 * self.rd)
    }

    // The name of variables below correspond to ``The Glicko system''
    fn q() -> f64 {
        10_f64.ln() / 400.0
    }

    fn g(&self) -> f64 {
        let t = 1.0 + 3.0 * Self::q().powf(2.0) * self.rd.powf(2.0) / PI.powf(2.0);
        1.0 / t.sqrt()
    }

    fn e(&self, r: f64) -> f64 {
        let t = -self.g() * (r - self.rating) / 400.0;
        1.0 / (1.0 + 10_f64.powf(t))
    }
}

fn main() {
    {
        // (1) Example of a rating update

        // Player to be updated
        let mut main_player = Player {
            rating: 1500.0,
            rd: 200.0,
        };

        // 3 opponets in a rating period
        let p1 = Player {
            rating: 1400.0,
            rd: 30.0,
        };
        let p2 = Player {
            rating: 1550.0,
            rd: 100.0,
        };
        let p3 = Player {
            rating: 1700.0,
            rd: 300.0,
        };

        let game_results = vec![
            GameResult(p1.clone(), Outcome::Win),
            GameResult(p2.clone(), Outcome::Lose),
            GameResult(p3.clone(), Outcome::Lose),
        ];

        main_player.update(&game_results);

        println!(
            "rating: {:.4}, RD: {:.4}",
            main_player.rating, main_player.rd
        );
    }

    {
        // (2) Example of a expected outcome of a game between two players
        let p1 = Player {
            rating: 1400.0,
            rd: 80.0,
        };
        let p2 = Player {
            rating: 1500.0,
            rd: 150.0,
        };

        println!(
            "probability that p1 beats p2: {:.3}",
            p1.expected_outcome(&p2)
        );
    }

    {
        // (3) Example of getting a 95 % confidence interaval for a player
        let p = Player {
            rating: 1500.0,
            rd: 30.0,
        };

        let interaval = p.get_95confidence_interval();
        println!(
            "95% confidence interaval: ({}, {})",
            interaval.0, interaval.1
        );
    }
}

use std::io;
use std::time::{Duration, Instant};

use blackjack::{
    card::{Card, CardValue, Hand},
    game::{
        Outcome::{self, DealerBust, DealerWin, PlayerBust, PlayerWin, Push},
        Stage::{self, DealerDraws, DealingCards, GameEnd, PlayerDecision},
    },
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use rand::RngExt;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

const DEALER_DRAW_DELAY: Duration = Duration::from_millis(500);

fn main() -> io::Result<()> {
    colored::control::set_override(true);
    ratatui::run(|terminal| App::default().run(terminal))
}

#[derive(Debug)]
pub struct App {
    stage: Stage,
    exit: bool,
    outcome: Outcome,
    selected_bet: i32,
    money: i32,
    dealer: Option<Hand>,
    player: Option<Hand>,
    split_player: Option<Hand>,
    active_hand: usize,
    split_results: Option<[Outcome; 2]>,
    next_dealer_draw: Option<Instant>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            stage: Stage::default(),
            exit: false,
            outcome: Outcome::default(),
            selected_bet: 25,
            money: 500,
            dealer: None,
            player: None,
            split_player: None,
            active_hand: 0,
            split_results: None,
            next_dealer_draw: None,
        }
    }
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
            self.handle_game_loop();
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> io::Result<()> {
        if self.stage == DealerDraws
            && !event::poll(
                self.next_dealer_draw
                    .unwrap_or_else(Instant::now)
                    .saturating_duration_since(Instant::now()),
            )?
        {
            self.advance_dealer();
            self.next_dealer_draw = (self.stage == DealerDraws)
                .then(|| Instant::now() + DEALER_DRAW_DELAY);
            return Ok(());
        }

        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event);
                self.handle_game_loop();
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_game_loop(&mut self) {
        match self.stage {
            Stage::Betting => {}
            Stage::DealingCards => {
                self.dealer = Some(self.deal_hand(true));
                self.player = Some(self.deal_hand(false));
                self.split_player = None;
                self.active_hand = 0;
                self.split_results = None;

                self.stage = PlayerDecision;
            }
            Stage::PlayerDecision => {
            }
            Stage::DealerDraws => {
                self.dealer.as_mut().unwrap().cards[1].face_up = true;
            }
            Stage::GameEnd => {

            }
        }
    }

    fn advance_dealer(&mut self) {
        let dealer = self.dealer.as_ref().unwrap();

        if self.check_count(dealer) {
            if self.split_player.is_some() {
                self.finish_split_round(true);
            } else {
                self.finish_round(DealerBust);
            }
            return;
        }

        let dealer_total = self.hand_total(dealer);
        if self.check_17(dealer) {
            if self.split_player.is_some() {
                self.finish_split_round(false);
            } else {
                let player_total = self.hand_total(self.player.as_ref().unwrap());
                let outcome = match dealer_total.cmp(&player_total) {
                    std::cmp::Ordering::Greater => DealerWin,
                    std::cmp::Ordering::Less => PlayerWin,
                    std::cmp::Ordering::Equal => Push,
                };
                self.finish_round(outcome);
            }
        } else {
            self.draw_card(true);
        }
    }

    fn finish_round(&mut self, outcome: Outcome) {
        if self.stage == GameEnd {
            return;
        }

        let bet = self.player.as_ref().unwrap().bet;
        match outcome {
            PlayerWin | DealerBust => self.money += bet * 2,
            Push => self.money += bet,
            PlayerBust | DealerWin => {}
        }

        self.outcome = outcome;
        self.stage = GameEnd;
        self.next_dealer_draw = None;
    }

    fn finish_split_round(&mut self, dealer_bust: bool) {
        let dealer_total = self.hand_total(self.dealer.as_ref().unwrap());
        let first = self.player.as_ref().unwrap();
        let second = self.split_player.as_ref().unwrap();
        let results = [
            self.hand_outcome(first, dealer_total, dealer_bust),
            self.hand_outcome(second, dealer_total, dealer_bust),
        ];

        for (hand, result) in [(first, results[0]), (second, results[1])] {
            match result {
                PlayerWin | DealerBust => self.money += hand.bet * 2,
                Push => self.money += hand.bet,
                PlayerBust | DealerWin => {}
            }
        }

        self.split_results = Some(results);
        self.outcome = results[0];
        self.stage = GameEnd;
        self.next_dealer_draw = None;
    }

    fn hand_outcome(&self, hand: &Hand, dealer_total: i32, dealer_bust: bool) -> Outcome {
        if self.check_count(hand) {
            PlayerBust
        } else if dealer_bust {
            DealerBust
        } else {
            match dealer_total.cmp(&self.hand_total(hand)) {
                std::cmp::Ordering::Greater => DealerWin,
                std::cmp::Ordering::Less => PlayerWin,
                std::cmp::Ordering::Equal => Push,
            }
        }
    }

    fn deal_hand(&mut self, is_dealer: bool) -> Hand {
        let mut rng = rand::rng();
        let val1 = rng.random_range(1..=10);
        let val2 = rng.random_range(1..=10);

        let suit1 = rng.random_range(1..=4);
        let suit2 = rng.random_range(1..=4);

        let card1 = Card::new(val1, suit1,true);
        let card2 = Card::new(val2, suit2,!is_dealer);

        Hand {
            cards: vec![card1, card2],
            bet: if is_dealer { 0 } else { self.selected_bet },
        }
    }

    fn draw_card(&mut self, is_dealer: bool) {
        let mut rng = rand::rng();
        let val = rng.random_range(1..=10);
        let suit = rng.random_range(1..=4);
        let card = Card::new(val, suit,true);

        if is_dealer {
            self.dealer.as_mut().unwrap().cards.push(card);
        } else {
            self.active_player_hand_mut().cards.push(card);
        }
    }

    fn active_player_hand(&self) -> &Hand {
        if self.active_hand == 0 {
            self.player.as_ref().unwrap()
        } else {
            self.split_player.as_ref().unwrap()
        }
    }

    fn active_player_hand_mut(&mut self) -> &mut Hand {
        if self.active_hand == 0 {
            self.player.as_mut().unwrap()
        } else {
            self.split_player.as_mut().unwrap()
        }
    }

    fn start_dealer_draws(&mut self) {
        self.stage = DealerDraws;
        self.next_dealer_draw = Some(Instant::now() + DEALER_DRAW_DELAY);
    }

    fn finish_player_hand(&mut self) {
        if self.split_player.is_some() && self.active_hand == 0 {
            self.active_hand = 1;
            self.stage = PlayerDecision;
        } else if self.split_player.is_some()
            && self.check_count(self.player.as_ref().unwrap())
            && self.check_count(self.split_player.as_ref().unwrap())
        {
            self.finish_split_round(false);
        } else {
            self.start_dealer_draws();
        }
    }

    fn player_busted(&mut self) {
        if self.split_player.is_none() {
            self.finish_round(PlayerBust);
        } else if self.active_hand == 0 {
            self.finish_player_hand();
        } else if self.check_count(self.player.as_ref().unwrap()) {
            self.finish_split_round(false);
        } else {
            self.finish_player_hand();
        }
    }

    fn split(&mut self) {
        let Some(hand) = self.player.as_ref() else {
            return;
        };
        if self.split_player.is_some()
            || hand.cards.len() != 2
            || self.money < hand.bet
            || self.card_value_to_i32(hand.cards[0].value)
                != self.card_value_to_i32(hand.cards[1].value)
        {
            return;
        }

        let bet = hand.bet;
        let is_aces = self.card_value_to_i32(hand.cards[0].value) == 11;
        let mut cards = std::mem::take(&mut self.player.as_mut().unwrap().cards);
        let first = cards.remove(0);
        let second = cards.remove(0);
        self.player.as_mut().unwrap().cards.push(first);
        self.split_player = Some(Hand {
            cards: vec![second],
            bet,
        });
        self.money -= bet;
        self.active_hand = 0;
        self.split_results = None;
        self.draw_card(false);
        self.active_hand = 1;
        self.draw_card(false);
        self.active_hand = 0;

        if is_aces {
            self.start_dealer_draws();
        }
    }

    fn check_count(&self, hand: &Hand) -> bool {
        let mut count = 0;
        let mut aces = 0;

        for card in &hand.cards {
            count += match &card.value {
                CardValue::Ace => {
                    aces += 1;
                    1
                }
                CardValue::Two => 2,
                CardValue::Three => 3,
                CardValue::Four => 4,
                CardValue::Five => 5,
                CardValue::Six => 6,
                CardValue::Seven => 7,
                CardValue::Eight => 8,
                CardValue::Nine => 9,
                CardValue::Ten => 10,
            };
        }

        if aces > 0 && count + 10 <= 21 {
            count += 10;
        }

        count > 21
    }

    fn check_17(&self, hand: &Hand) -> bool {
        let mut count = 0;
        let mut aces = 0;

        for card in &hand.cards {
            count += match &card.value {
                CardValue::Ace => {
                    aces += 1;
                    1
                }
                CardValue::Two => 2,
                CardValue::Three => 3,
                CardValue::Four => 4,
                CardValue::Five => 5,
                CardValue::Six => 6,
                CardValue::Seven => 7,
                CardValue::Eight => 8,
                CardValue::Nine => 9,
                CardValue::Ten => 10,
            };
        }

        if aces > 0 && count + 10 <= 21 {
            count += 10;
        }

        count >= 17
    }

    fn hand_total(&self, hand: &Hand) -> i32 {
        let mut total = 0;
        let mut has_ace = false;

        for card in &hand.cards {
            total += match card.value {
                CardValue::Ace => {
                    has_ace = true;
                    1
                }
                CardValue::Two => 2,
                CardValue::Three => 3,
                CardValue::Four => 4,
                CardValue::Five => 5,
                CardValue::Six => 6,
                CardValue::Seven => 7,
                CardValue::Eight => 8,
                CardValue::Nine => 9,
                CardValue::Ten => 10,
            };
        }

        if has_ace && total + 10 <= 21 {
            total += 10;
        }

        total
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if self.stage == GameEnd && key_event.code == KeyCode::Char('1') {
            self.stage = Stage::Betting;
            self.dealer = None;
            self.player = None;
            self.split_player = None;
            self.active_hand = 0;
            self.split_results = None;
            self.outcome = Outcome::default();
            self.next_dealer_draw = None;
            return;
        }
        if self.stage == GameEnd && key_event.code != KeyCode::Char('q') {
            return;
        }
        if self.stage == DealerDraws && key_event.code != KeyCode::Char('q') {
            return;
        }

        if self.stage == Stage::Betting {
            match key_event.code {
                KeyCode::Char('1') => self.selected_bet = 25,
                KeyCode::Char('2') => self.selected_bet = 50,
                KeyCode::Char('3') => self.selected_bet = 100,
                KeyCode::Enter if self.selected_bet <= self.money => {
                    self.money -= self.selected_bet;
                    self.stage = DealingCards;
                }
                KeyCode::Enter => {}
                KeyCode::Char('q') => self.exit(),
                _ => {}
            }
            return;
        }

        match key_event.code {
            KeyCode::Char('1') => {
                self.stand()
            }
            KeyCode::Char('2') => {
                self.hit();

                if self.check_count(self.active_player_hand()) {
                    self.player_busted();
                }
            }
            KeyCode::Char('3') => {
                self.double()
            }
            KeyCode::Char('4') => self.split(),
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }

    fn card_value_to_i32(&self, value: CardValue) -> i32 {
        let out: i32;
        match value {
            CardValue::Ace => out = 11,
            CardValue::Two => out = 2,
            CardValue::Three => out = 3,
            CardValue::Four => out = 4,
            CardValue::Five => out = 5,
            CardValue::Six => out = 6,
            CardValue::Seven => out = 7,
            CardValue::Eight => out = 8,
            CardValue::Nine => out = 9,
            CardValue::Ten => out = 10,
        }

        out
    }

    fn hit(&mut self) {
        self.draw_card(false);
    }

    fn stand(&mut self) {
        self.finish_player_hand();
    }

    fn double(&mut self) {
        let bet = self.active_player_hand().bet;
        if self.money >= bet {
            self.money -= bet;
            self.active_player_hand_mut().bet *= 2;
            self.hit();
            if self.check_count(self.active_player_hand()) {
                self.player_busted();
            } else {
                self.stand();
            }
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" blackjack ".bold());
        let instructions = if self.stage == Stage::Betting {
            Line::from(" Choose bet: 1 = 25, 2 = 50, 3 = 100 | Enter to deal | q to quit ")
        } else {
            Line::from(vec![
                " Stand ".into(),
                "<1>".blue().bold(),
                " Hit ".into(),
                "<2>".blue().bold(),
                " Double ".into(),
                "<3> ".blue().bold(),
                " Split pair ".into(),
                "<4> ".blue().bold(),
            ])
        };

        let line = match self.outcome {
            DealerWin => Line::from(
                " Dealer Wins! Press 1 to play again or q to quit "
                    .red()
                    .bold(),
            ),
            PlayerWin => Line::from(
                " Player Wins! Press 1 to play again or q to quit "
                    .green()
                    .bold(),
            ),
            DealerBust => Line::from(
                " Dealer Bust! Press 1 to play again or q to quit "
                    .green()
                    .bold(),
            ),
            PlayerBust => Line::from(" BUST! Press 1 to play again or q to quit ".red().bold()),
            Push => Line::from(" Push! Press 1 to play again or q to quit ".bold()),
        };
        let line = if self.split_results.is_some() {
            Line::from(" Split round complete! Press 1 to play again or q to quit ".bold())
        } else {
            line
        };

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(if self.stage == GameEnd {
                line
            } else {
                instructions.centered()
            })
            .border_set(border::THICK);

        let inner = block.inner(area);
        block.render(area, buf);

        if self.stage == Stage::Betting {
            let status = if self.selected_bet > self.money {
                "\nNot enough money for this bet"
            } else {
                ""
            };
            Paragraph::new(format!(
                "Money: ${}\nSelected bet: ${}{}",
                self.money, self.selected_bet, status
            ))
                .centered()
                .render(inner, buf);
            return;
        }

        if self.stage != PlayerDecision
            && self.stage != DealerDraws
            && self.stage != GameEnd
        {
            return;
        }

        let sections = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(9),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(9),
        ])
        .split(inner);

        let mut dealer_card_value = self.card_value_to_i32(self.dealer.as_ref().unwrap().cards[0].value);
        if self.stage == DealerDraws || self.stage == GameEnd {
            dealer_card_value = self.hand_total(self.dealer.as_ref().unwrap());
        }
        let player_card_value = self.hand_total(self.player.as_ref().unwrap());
        let player_bet = self.player.as_ref().unwrap().bet;

        Paragraph::new(format!("Dealer Cards: {}", dealer_card_value))
            .centered()
            .render(sections[0], buf);
        render_hand(self.dealer.as_ref().unwrap(), sections[1], buf);
        if let Some(split_hand) = &self.split_player {
            let hand_status = if self.stage == GameEnd {
                "Split results".to_string()
            } else {
                format!(
                    "Hand {} is active",
                    if self.active_hand == 0 { "1" } else { "2" }
                )
            };
            Paragraph::new(format!("{} | Money: ${}", hand_status, self.money))
                .centered()
                .render(sections[2], buf);

            let hand_areas = Layout::horizontal([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(sections[3]);
            let card_areas = Layout::horizontal([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(sections[4]);
            let results = self.split_results;
            let first_result = results.map(|value| outcome_label(value[0]));
            let second_result = results.map(|value| outcome_label(value[1]));
            Paragraph::new(format!(
                "Hand 1: {} Bet: ${}{}",
                self.hand_total(self.player.as_ref().unwrap()),
                player_bet,
                first_result.map(|text| format!(" - {}", text)).unwrap_or_default()
            ))
            .centered()
            .render(hand_areas[0], buf);
            Paragraph::new(format!(
                "Hand 2: {} Bet: ${}{}",
                self.hand_total(split_hand),
                split_hand.bet,
                second_result.map(|text| format!(" - {}", text)).unwrap_or_default()
            ))
            .centered()
            .render(hand_areas[1], buf);
            render_hand(self.player.as_ref().unwrap(), card_areas[0], buf);
            render_hand(split_hand, card_areas[1], buf);
        } else {
            Paragraph::new(format!(
                "Player Cards: {} Bet: ${} Money: ${}",
                player_card_value, player_bet, self.money
            ))
            .centered()
            .render(sections[3], buf);
            render_hand(self.player.as_ref().unwrap(), sections[4], buf);
        }
    }
}

fn outcome_label(outcome: Outcome) -> &'static str {
    match outcome {
        PlayerBust => "Bust",
        DealerBust | PlayerWin => "Win",
        DealerWin => "Loss",
        Push => "Push",
    }
}

fn render_hand(hand: &Hand, area: Rect, buf: &mut Buffer) {
    const CARD_WIDTH: u16 = 11;
    const CARD_SPACING: u16 = 1;

    let hand_width = hand
        .cards
        .iter()
        .fold(0u16, |width, _| width.saturating_add(CARD_WIDTH + CARD_SPACING))
        .saturating_sub(CARD_SPACING);

    let start_x = area
        .x
        .saturating_add(area.width.saturating_sub(hand_width) / 2);

    for (index, card) in hand.cards.iter().enumerate() {
        let lines = if card.face_up {
            card.card_to_ascii()
        } else {
            card.print_hidden()
        };

        let card_x = start_x.saturating_add(
            (index as u16).saturating_mul(CARD_WIDTH + CARD_SPACING),
        );

        // Define the exact sub-area for this specific card
        let card_area = Rect {
            x: card_x,
            y: area.y,
            width: CARD_WIDTH.min(area.right().saturating_sub(card_x)),
            height: (lines.len() as u16).min(area.height),
        };

        // Render the card's lines (with their Spans and colors intact) into the buffer
        Paragraph::new(lines).render(card_area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standing_reveals_dealers_hidden_card_and_finishes_at_17() {
        let mut app = App {
            dealer: Some(Hand {
                cards: vec![Card::new(10, 1, true), Card::new(7, 2, false)],
                bet: 0,
            }),
            player: Some(Hand {
                cards: vec![Card::new(10, 1, true), Card::new(7, 2, true)],
                bet: 0,
            }),
            ..App::default()
        };

        app.stand();
        app.handle_game_loop();

        let dealer = app.dealer.as_ref().unwrap();
        assert!(dealer.cards[1].face_up);
        assert_eq!(dealer.cards.len(), 2);
        assert_eq!(app.stage, DealerDraws);

        app.advance_dealer();
        assert_eq!(app.dealer.as_ref().unwrap().cards.len(), 2);
        assert_eq!(app.stage, GameEnd);
    }

    #[test]
    fn dealer_draws_one_card_per_advance() {
        let mut app = App {
            stage: DealerDraws,
            dealer: Some(Hand {
                cards: vec![Card::new(2, 1, true), Card::new(2, 2, true)],
                bet: 0,
            }),
            player: Some(Hand {
                cards: vec![Card::new(10, 1, true), Card::new(8, 2, true)],
                bet: 0,
            }),
            ..App::default()
        };

        app.advance_dealer();

        assert_eq!(app.dealer.as_ref().unwrap().cards.len(), 3);
        assert_eq!(app.stage, DealerDraws);
    }

    #[test]
    fn retry_starts_a_fresh_round() {
        let mut app = App {
            stage: GameEnd,
            outcome: PlayerBust,
            dealer: Some(Hand {
                cards: vec![Card::new(10, 1, true), Card::new(8, 2, true)],
                bet: 0,
            }),
            player: Some(Hand {
                cards: vec![Card::new(10, 1, true), Card::new(10, 2, true)],
                bet: 0,
            }),
            next_dealer_draw: Some(Instant::now()),
            ..App::default()
        };

        app.handle_key_event(KeyEvent::new(
            KeyCode::Char('1'),
            crossterm::event::KeyModifiers::NONE,
        ));

        assert_eq!(app.stage, Stage::Betting);
        assert_eq!(app.outcome, Outcome::default());
        assert!(app.dealer.is_none());
        assert!(app.player.is_none());
        assert!(app.next_dealer_draw.is_none());

        app.handle_game_loop();

        assert_eq!(app.stage, Stage::Betting);
        app.handle_key_event(KeyEvent::new(
            KeyCode::Enter,
            crossterm::event::KeyModifiers::NONE,
        ));
        app.handle_game_loop();

        assert_eq!(app.stage, PlayerDecision);
        assert_eq!(app.dealer.as_ref().unwrap().cards.len(), 2);
        assert_eq!(app.player.as_ref().unwrap().cards.len(), 2);
        assert_eq!(app.player.as_ref().unwrap().bet, 25);
        assert_eq!(app.money, 475);
    }

    #[test]
    fn betting_keys_select_amount_and_enter_starts_round() {
        let mut app = App::default();

        for (key, expected_bet) in [('1', 25), ('2', 50), ('3', 100)] {
            app.handle_key_event(KeyEvent::new(
                KeyCode::Char(key),
                crossterm::event::KeyModifiers::NONE,
            ));
            assert_eq!(app.selected_bet, expected_bet);
            assert_eq!(app.stage, Stage::Betting);
        }

        app.handle_key_event(KeyEvent::new(
            KeyCode::Enter,
            crossterm::event::KeyModifiers::NONE,
        ));
        assert_eq!(app.stage, DealingCards);
        assert_eq!(app.money, 400);
    }

    #[test]
    fn round_results_update_money_and_cannot_be_settled_twice() {
        let mut app = App {
            stage: PlayerDecision,
            money: 450,
            player: Some(Hand {
                cards: vec![Card::new(10, 1, true), Card::new(8, 2, true)],
                bet: 50,
            }),
            ..App::default()
        };

        app.finish_round(PlayerWin);
        assert_eq!(app.money, 550);

        app.finish_round(PlayerWin);
        assert_eq!(app.money, 550);

        app.stage = PlayerDecision;
        app.money = 450;
        app.finish_round(DealerWin);
        assert_eq!(app.money, 450);

        app.stage = PlayerDecision;
        app.finish_round(Push);
        assert_eq!(app.money, 500);
    }

    #[test]
    fn double_down_only_uses_money_when_the_extra_bet_is_affordable() {
        let mut app = App {
            stage: PlayerDecision,
            money: 25,
            player: Some(Hand {
                cards: vec![Card::new(2, 1, true), Card::new(3, 2, true)],
                bet: 50,
            }),
            ..App::default()
        };

        app.double();
        assert_eq!(app.money, 25);
        assert_eq!(app.player.as_ref().unwrap().bet, 50);
        assert_eq!(app.stage, PlayerDecision);

        app.money = 50;
        app.double();
        assert_eq!(app.money, 0);
        assert_eq!(app.player.as_ref().unwrap().bet, 100);
        assert_eq!(app.stage, DealerDraws);
    }

    #[test]
    fn splitting_requires_a_pair_and_enough_money_then_plays_both_hands() {
        let mut app = App {
            stage: PlayerDecision,
            money: 24,
            player: Some(Hand {
                cards: vec![Card::new(8, 1, true), Card::new(9, 2, true)],
                bet: 25,
            }),
            ..App::default()
        };

        app.split();
        assert!(app.split_player.is_none());

        app.player.as_mut().unwrap().cards[1] = Card::new(8, 2, true);
        app.split();
        assert!(app.split_player.is_none());

        app.money = 50;
        app.split();
        assert_eq!(app.money, 25);
        assert_eq!(app.player.as_ref().unwrap().bet, 25);
        assert_eq!(app.split_player.as_ref().unwrap().bet, 25);
        assert_eq!(app.player.as_ref().unwrap().cards.len(), 2);
        assert_eq!(app.split_player.as_ref().unwrap().cards.len(), 2);
        assert_eq!(app.active_hand, 0);

        app.stand();
        assert_eq!(app.stage, PlayerDecision);
        assert_eq!(app.active_hand, 1);
        app.stand();
        assert_eq!(app.stage, DealerDraws);
    }

    #[test]
    fn split_results_settle_each_hand_independently() {
        let mut app = App {
            stage: PlayerDecision,
            money: 0,
            dealer: Some(Hand {
                cards: vec![Card::new(10, 1, true), Card::new(8, 2, true)],
                bet: 0,
            }),
            player: Some(Hand {
                cards: vec![Card::new(10, 1, true), Card::new(9, 2, true)],
                bet: 25,
            }),
            split_player: Some(Hand {
                cards: vec![Card::new(10, 3, true), Card::new(7, 4, true)],
                bet: 25,
            }),
            ..App::default()
        };

        app.finish_split_round(false);

        assert_eq!(app.money, 50);
        assert_eq!(app.split_results, Some([PlayerWin, DealerWin]));
        assert_eq!(app.stage, GameEnd);
    }

    #[test]
    fn splitting_aces_deals_one_card_to_each_hand_and_ends_player_turn() {
        let mut app = App {
            stage: PlayerDecision,
            money: 25,
            player: Some(Hand {
                cards: vec![Card::new(1, 1, true), Card::new(1, 2, true)],
                bet: 25,
            }),
            ..App::default()
        };

        app.split();

        assert_eq!(app.stage, DealerDraws);
        assert_eq!(app.player.as_ref().unwrap().cards.len(), 2);
        assert_eq!(app.split_player.as_ref().unwrap().cards.len(), 2);
        assert_eq!(app.money, 0);
    }
}

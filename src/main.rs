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
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

const DEALER_DRAW_DELAY: Duration = Duration::from_millis(500);

fn main() -> io::Result<()> {
    colored::control::set_override(true);
    ratatui::run(|terminal| App::default().run(terminal))
}

#[derive(Debug, Default)]
pub struct App {
    stage: Stage,
    exit: bool,
    player_bet: i32,
    outcome: Outcome,
    dealer: Option<Hand>,
    player: Option<Hand>,
    next_dealer_draw: Option<Instant>,
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
            Stage::Betting => {

                self.stage = DealingCards;
            }
            Stage::DealingCards => {
                self.dealer = Some(self.deal_hand(true));
                self.player = Some(self.deal_hand(false));

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
            self.outcome = DealerBust;
            self.stage = GameEnd;
            self.next_dealer_draw = None;
            return;
        }

        let dealer_total = self.hand_total(dealer);
        let player_total = self.hand_total(self.player.as_ref().unwrap());

        if self.check_17(dealer) {
            self.outcome = match dealer_total.cmp(&player_total) {
                std::cmp::Ordering::Greater => DealerWin,
                std::cmp::Ordering::Less => PlayerWin,
                std::cmp::Ordering::Equal => Push,
            };
            self.stage = GameEnd;
            self.next_dealer_draw = None;
        } else {
            self.draw_card(true);
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
            self.player.as_mut().unwrap().cards.push(card);
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
            self.stage = DealingCards;
            self.dealer = None;
            self.player = None;
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

        match key_event.code {
            KeyCode::Char('1') => {
                self.stand()
            }
            KeyCode::Char('2') => {
                self.hit();

                if self.check_count(self.player.as_ref().unwrap()) {
                    self.outcome = PlayerBust;
                    self.stage = GameEnd;
                }
            }
            KeyCode::Char('3') => {
                self.hit()
            }
            KeyCode::Char('4') => {
                self.hit()
            }
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
        self.stage = DealerDraws;
        self.next_dealer_draw = Some(Instant::now() + DEALER_DRAW_DELAY);
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" blackjack ".bold());
        let instructions = Line::from(vec![
            " Stand ".into(),
            "<1>".blue().bold(),
            " Hit ".into(),
            "<2>".blue().bold(),
            " Double ".into(),
            "<3> ".blue().bold(),
            " Split ".into(),
            "<4> ".blue().bold(),
        ]);

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
            Constraint::Length(9),
        ])
        .split(inner);

        let mut dealer_card_value = self.card_value_to_i32(self.dealer.as_ref().unwrap().cards[0].value);
        if self.stage == DealerDraws || self.stage == GameEnd {
            dealer_card_value = self.hand_total(self.dealer.as_ref().unwrap());
        }
        let player_card_value = self.hand_total(self.player.as_ref().unwrap());

        Paragraph::new(format!("Dealer Cards: {}", dealer_card_value))
            .centered()
            .render(sections[0], buf);
        render_hand(self.dealer.as_ref().unwrap(), sections[1], buf);
        Paragraph::new(format!("Player Cards: {}", player_card_value))
            .centered()
            .render(sections[2], buf);
        render_hand(self.player.as_ref().unwrap(), sections[3], buf);
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
            }),
            player: Some(Hand {
                cards: vec![Card::new(10, 1, true), Card::new(7, 2, true)],
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
            }),
            player: Some(Hand {
                cards: vec![Card::new(10, 1, true), Card::new(8, 2, true)],
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
            }),
            player: Some(Hand {
                cards: vec![Card::new(10, 1, true), Card::new(10, 2, true)],
            }),
            next_dealer_draw: Some(Instant::now()),
            ..App::default()
        };

        app.handle_key_event(KeyEvent::new(
            KeyCode::Char('1'),
            crossterm::event::KeyModifiers::NONE,
        ));

        assert_eq!(app.stage, DealingCards);
        assert_eq!(app.outcome, Outcome::default());
        assert!(app.dealer.is_none());
        assert!(app.player.is_none());
        assert!(app.next_dealer_draw.is_none());

        app.handle_game_loop();

        assert_eq!(app.stage, PlayerDecision);
        assert_eq!(app.dealer.as_ref().unwrap().cards.len(), 2);
        assert_eq!(app.player.as_ref().unwrap().cards.len(), 2);
    }
}

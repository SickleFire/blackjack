use std::io;
use std::thread::sleep;
use std::time::Duration;

use blackjack::{
    card::{Card, CardValue, Hand}, game::Stage::{self, DealerDraws, DealingCards, GameEnd, PlayerDecision},
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use rand::{Rng, RngExt};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}

#[derive(Debug, Default)]
pub struct App {
    stage: Stage,
    exit: bool,
    player_bet: i32,
    dealer: Option<Hand>,
    player: Option<Hand>,
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

            }
            Stage::GameEnd => {

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

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('1') => {
                self.stand()
            }
            KeyCode::Char('2') => {
                self.hit();

                sleep(Duration::from_millis(500));

                if self.check_count(self.player.as_ref().unwrap()) {
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

    fn hit(&mut self) {
        self.draw_card(false);
    }

    fn stand(&mut self) {
        self.stage = DealerDraws;
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
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let inner = block.inner(area);
        block.render(area, buf);

        if self.stage != PlayerDecision {
            return;
        }

        let sections = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(9),
            Constraint::Length(1),
            Constraint::Length(9),
        ])
        .split(inner);

        Paragraph::new("Dealer Cards:")
            .centered()
            .render(sections[0], buf);
        render_hand(self.dealer.as_ref().unwrap(), sections[1], buf);
        Paragraph::new("Player Cards:")
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
        let visible_width = CARD_WIDTH.min(area.right().saturating_sub(card_x)) as usize;

        for (row, line) in lines.iter().enumerate().take(area.height as usize) {
            buf.set_stringn(
                card_x,
                area.y.saturating_add(row as u16),
                line,
                visible_width,
                Style::default(),
            );
        }
    }
}

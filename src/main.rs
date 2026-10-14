use std::io;

use blackjack::{
    card::{Card, Hand},
    game::Stage::{self, PlayerDecision},
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use rand::{Rng, RngExt};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Widget},
};

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}

#[derive(Debug, Default)]
pub struct App {
    stage: Stage,
    exit: bool,
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
            Stage::DealingCards => {
                self.dealer = Some(self.deal_hand(true));
                self.player = Some(self.deal_hand(false));

                self.stage = PlayerDecision;
            }
            Stage::PlayerDecision => {}
            Stage::DealerDraws => {}
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

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
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

        let player_cards = if self.stage == PlayerDecision {
            let player_card1 = self.player.as_ref().unwrap().cards[0].value.to_string();
            let player_card2 = self.player.as_ref().unwrap().cards[1].value.to_string();

            let dealer_card1 = self.dealer.as_ref().unwrap().cards[0].value.to_string();
            let hidden_card = self.dealer.as_ref().unwrap().cards[1].print_hidden();
            let mut lines = vec![Line::from(vec![
                Span::raw("Dealer Cards: "),
                Span::raw(dealer_card1),
            ])];
            lines.extend(hidden_card.into_iter().map(Line::from));
            lines.push(Line::from(vec![
                Span::raw("Player Cards: "),
                Span::raw(player_card1),
                Span::raw(" "),
                Span::raw(player_card2),
            ]));
            Text::from(lines)
        } else {
            Text::from("")
        };

        Paragraph::new(player_cards)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

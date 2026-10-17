use crate::card::{CardValue::{Ace, Eight, Five, Four, Nine, Seven, Six, Ten, Three, Two}, Suit::{Diamonds, Hearts, Clubs, Spades}};
use ratatui::{
    text::{Line, Span},
    style::{Color, Style},
};
use std::fmt;
#[derive(Debug, Copy, Clone)]
pub enum CardValue {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
}

impl fmt::Display for CardValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let value = match self {
            CardValue::Ace => "A",
            CardValue::Two => "2",
            CardValue::Three => "3",
            CardValue::Four => "4",
            CardValue::Five => "5",
            CardValue::Six => "6",
            CardValue::Seven => "7",
            CardValue::Eight => "8",
            CardValue::Nine => "9",
            CardValue::Ten => "10",
        };

        f.pad(value)
    }
}

#[derive(Debug, PartialEq)]
pub enum Suit {
    Hearts,
    Clubs,
    Diamonds,
    Spades,
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let symbol = match self {
            Suit::Hearts => "♥",
            Suit::Clubs => "♣",
            Suit::Diamonds => "♦",
            Suit::Spades => "♠",
        };

        f.pad(symbol)
    }
}

#[derive(Debug)]
pub struct Hand {
    pub cards: Vec<Card>,
}

#[derive(Debug)]
pub struct Card {
    pub value: CardValue,
    pub suit: Suit,
    pub face_up: bool,
}

impl Card {
    pub fn new(value: i32, suit: i32, face_up: bool) -> Self {
        let value = match value {
            1 => Ace,
            2 => Two,
            3 => Three,
            4 => Four,
            5 => Five,
            6 => Six,
            7 => Seven,
            8 => Eight,
            9 => Nine,
            10 => Ten,
            _ => panic!("Unsupported card value: {}", value),
        };

        let suit = match suit {
            1 => Hearts,
            2 => Clubs,
            3 => Diamonds,
            4 => Spades,
            _ => panic!("Unsupported suit value: {}", suit),
        };

        Self { value, suit, face_up }
    }

pub fn card_to_ascii(&self) -> Vec<Line<'static>> {
        let is_red = self.suit == Diamonds || self.suit == Hearts;
        let suit_color = if is_red { Color::Red } else { Color::Reset };

        let top_val = format!("{:<9}", self.value);
        let mid_suit = format!("{:^9}", self.suit);
        let bot_val = format!("{:>9}", self.value);

        vec![
            Line::from("┌─────────┐"),
            Line::from(vec![
                Span::raw("│"),
                Span::styled(top_val, Style::default()),
                Span::raw("│"),
            ]),
            Line::from("│         │"),
            Line::from("│         │"),
            Line::from(vec![
                Span::raw("│"),
                Span::styled(mid_suit, Style::default().fg(suit_color)),
                Span::raw("│"),
            ]),
            Line::from("│         │"),
            Line::from("│         │"),
            Line::from(vec![
                Span::raw("│"),
                Span::styled(bot_val, Style::default()),
                Span::raw("│"),
            ]),
            Line::from("└─────────┘"),
        ]
    }

    pub fn print_hidden(&self) -> Vec<Line<'static>> {
        vec![
            Line::from("┌─────────┐"),
            Line::from("│░░░░░░░░░│"),
            Line::from("│░░░░░░░░░│"),
            Line::from("│░░░░░░░░░│"),
            Line::from("│░░░░░░░░░│"),
            Line::from("│░░░░░░░░░│"),
            Line::from("│░░░░░░░░░│"),
            Line::from("│░░░░░░░░░│"),
            Line::from("└─────────┘"),
        ]
    }
}

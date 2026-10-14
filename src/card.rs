use crate::card::{CardValue::{Ace, Eight, Five, Four, Nine, Seven, Six, Ten, Three, Two}, Suit::{Diamonds, Hearts, Clubs, Spades}};
use std::fmt;

#[derive(Debug)]
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
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[derive(Debug)]
pub enum Suit {
    Hearts,
    Clubs,
    Diamonds,
    Spades,
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
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

    pub fn card_to_ascii(&self) -> Vec<String> {
        let card = vec![];

        card
    }

    pub fn print_hidden(&self) -> Vec<String> {
        let card = vec![
            "┌─────────┐".to_string(),
            "│░░░░░░░░░│".to_string(),
            "│░░░░░░░░░│".to_string(),
            "│░░░░░░░░░│".to_string(),
            "│░░░░░░░░░│".to_string(),
            "│░░░░░░░░░│".to_string(),
            "│░░░░░░░░░│".to_string(),
            "│░░░░░░░░░│".to_string(),
            "└─────────┘".to_string(),
        ];

        card
    }
}



#[derive(Debug, Default, PartialEq)]
pub enum Stage {
    #[default]
    Betting,
    DealingCards,
    PlayerDecision,
    DealerDraws,
    GameEnd,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Outcome {
    PlayerBust,
    #[default]
    DealerBust,
    PlayerWin,
    DealerWin,
    Push,
}

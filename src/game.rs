#[derive(Debug, Default, PartialEq)]
pub enum Stage {
    #[default]
    Betting,
    DealingCards,
    PlayerDecision,
    DealerDraws,
    GameEnd,
}

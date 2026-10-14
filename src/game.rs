#[derive(Debug, Default, PartialEq)]
pub enum Stage {
    #[default]
    DealingCards,
    PlayerDecision,
    DealerDraws,
}

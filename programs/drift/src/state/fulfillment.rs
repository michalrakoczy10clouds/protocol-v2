use solana_program::pubkey::Pubkey;

// 10c BUY/SELL I think we don't need AMM - we want to remove it anyway
#[derive(Debug, PartialEq, Eq)]
pub enum PerpFulfillmentMethod {
    AMM(Option<u64>),
    Match(Pubkey, u16),
}

#[derive(Debug)]
pub enum SpotFulfillmentMethod {
    ExternalMarket,
    Match(Pubkey, u16),
}

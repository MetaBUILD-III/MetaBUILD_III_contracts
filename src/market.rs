use crate::*;

#[near_bindgen]
impl Contract {
    #[private]
    pub fn add_pair(&mut self, pairData: TradePair) {
        let pair = (pairData.sell_token.clone(), pairData.buy_token.clone());
        self.supported_markets.insert(&pair, &pairData);
    }

    #[private]
    pub fn remove_pair(&mut self, pairData: TradePair) {
        let pair = (pairData.sell_token.clone(), pairData.buy_token);
        self.supported_markets.remove(&pair);
    }
}

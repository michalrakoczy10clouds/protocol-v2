#[cfg(test)]
use crate::state::oracle_map::OracleMap;
use crate::state::state::FeeStructure;
use crate::state::user::{MarketPosition, Order};
use anchor_lang::prelude::Pubkey;
use anchor_lang::Owner;

fn get_fee_structure() -> FeeStructure {
    FeeStructure {
        fee_numerator: 5,
        fee_denominator: 10000,
        maker_rebate_numerator: 3,
        maker_rebate_denominator: 5,
        ..FeeStructure::default()
    }
}

fn get_user_keys() -> (Pubkey, Pubkey, Pubkey) {
    (Pubkey::default(), Pubkey::default(), Pubkey::default())
}

fn get_oracle_map<'a>() -> OracleMap<'a> {
    OracleMap::empty()
}

pub mod fulfill_order_with_maker_order {
    use super::*;
    use crate::controller::orders::fulfill_order_with_match;
    use crate::controller::position::PositionDirection;
    use crate::math::constants::{
        BASE_PRECISION, BASE_PRECISION_I128, MARK_PRICE_PRECISION, QUOTE_PRECISION,
    };
    use crate::state::market::Market;
    use crate::state::state::FeeStructure;
    use crate::state::user::{MarketPosition, Order, OrderType, User};
    use crate::tests::utils::*;

    #[test]
    fn long_taker_order_fulfilled_start_of_auction() {
        let mut taker = User {
            orders: get_orders(Order {
                market_index: 0,
                order_type: OrderType::Market,
                direction: PositionDirection::Long,
                base_asset_amount: BASE_PRECISION,
                ts: 0,
                slot: 0,
                auction_start_price: 100 * MARK_PRICE_PRECISION,
                auction_end_price: 200 * MARK_PRICE_PRECISION,
                auction_duration: 5,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_bids: BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            ..User::default()
        };

        let mut maker = User {
            orders: get_orders(Order {
                market_index: 0,
                post_only: true,
                order_type: OrderType::Limit,
                direction: PositionDirection::Short,
                base_asset_amount: BASE_PRECISION,
                ts: 0,
                price: 100 * MARK_PRICE_PRECISION,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_asks: -BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            ..User::default()
        };

        let mut market = Market::default();

        let now = 1_i64;
        let slot = 1_u64;

        let fee_structure = get_fee_structure();

        let (taker_key, maker_key, filler_key) = get_user_keys();

        let mut order_records = vec![];

        fulfill_order_with_match(
            &mut market,
            &mut taker,
            0,
            &taker_key,
            &mut maker,
            0,
            &maker_key,
            None,
            &filler_key,
            now,
            slot,
            &fee_structure,
            &mut get_oracle_map(),
            &mut order_records,
        )
        .unwrap();

        let taker_position = &taker.positions[0];
        assert_eq!(taker_position.base_asset_amount, BASE_PRECISION_I128);
        assert_eq!(taker_position.quote_asset_amount, 100 * QUOTE_PRECISION);
        assert_eq!(taker_position.quote_entry_amount, 100 * QUOTE_PRECISION);
        assert_eq!(taker_position.unsettled_pnl, -50000);
        assert_eq!(taker_position.open_bids, 0);
        assert_eq!(taker_position.open_orders, 0);
        assert_eq!(taker.fees.total_fee_paid, 50000);
        assert_eq!(taker.fees.total_referee_discount, 0);
        assert_eq!(taker.fees.total_token_discount, 0);
        assert_eq!(taker.orders[0], Order::default());

        let maker_position = &maker.positions[0];
        assert_eq!(maker_position.base_asset_amount, -BASE_PRECISION_I128);
        assert_eq!(maker_position.quote_asset_amount, 100 * QUOTE_PRECISION);
        assert_eq!(maker_position.quote_entry_amount, 100 * QUOTE_PRECISION);
        assert_eq!(maker_position.unsettled_pnl, 30000);
        assert_eq!(maker_position.open_orders, 0);
        assert_eq!(maker_position.open_asks, 0);
        assert_eq!(maker.fees.total_fee_rebate, 30000);
        assert_eq!(maker.orders[0], Order::default());

        assert_eq!(market.amm.net_base_asset_amount, 0);
        assert_eq!(market.base_asset_amount_long, BASE_PRECISION_I128);
        assert_eq!(market.base_asset_amount_short, -BASE_PRECISION_I128);
        assert_eq!(market.amm.quote_asset_amount_long, 100 * QUOTE_PRECISION);
        assert_eq!(market.amm.quote_asset_amount_short, 100 * QUOTE_PRECISION);
        assert_eq!(market.amm.total_fee, 20000);
        assert_eq!(market.amm.total_fee_minus_distributions, 20000);
        assert_eq!(market.amm.net_revenue_since_last_funding, 20000);
        assert_eq!(market.unsettled_profit, 30000);
        assert_eq!(market.unsettled_loss, 50000);
    }

    #[test]
    fn long_taker_order_fulfilled_middle_of_auction() {
        let mut taker = User {
            orders: get_orders(Order {
                market_index: 0,
                order_type: OrderType::Market,
                direction: PositionDirection::Long,
                base_asset_amount: BASE_PRECISION,
                ts: 0,
                slot: 0,
                auction_start_price: 100 * MARK_PRICE_PRECISION,
                auction_end_price: 200 * MARK_PRICE_PRECISION,
                auction_duration: 5,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_bids: BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            ..User::default()
        };

        let mut maker = User {
            orders: get_orders(Order {
                market_index: 0,
                post_only: true,
                order_type: OrderType::Limit,
                direction: PositionDirection::Short,
                base_asset_amount: BASE_PRECISION,
                ts: 0,
                price: 160 * MARK_PRICE_PRECISION,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_asks: -BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            ..User::default()
        };

        let mut market = Market::default();

        let now = 3_i64;
        let slot = 3_u64;

        let fee_structure = get_fee_structure();

        let (taker_key, maker_key, filler_key) = get_user_keys();

        let mut order_records = vec![];

        fulfill_order_with_match(
            &mut market,
            &mut taker,
            0,
            &taker_key,
            &mut maker,
            0,
            &maker_key,
            None,
            &filler_key,
            now,
            slot,
            &fee_structure,
            &mut get_oracle_map(),
            &mut order_records,
        )
        .unwrap();

        let taker_position = &taker.positions[0];
        assert_eq!(taker_position.base_asset_amount, BASE_PRECISION_I128);
        assert_eq!(taker_position.quote_asset_amount, 160 * QUOTE_PRECISION);
        assert_eq!(taker_position.quote_entry_amount, 160 * QUOTE_PRECISION);
        assert_eq!(taker_position.unsettled_pnl, -80000);
        assert_eq!(taker_position.open_bids, 0);
        assert_eq!(taker_position.open_orders, 0);
        assert_eq!(taker.fees.total_fee_paid, 80000);
        assert_eq!(taker.fees.total_referee_discount, 0);
        assert_eq!(taker.fees.total_token_discount, 0);
        assert_eq!(taker.orders[0], Order::default());

        let maker_position = &maker.positions[0];
        assert_eq!(maker_position.base_asset_amount, -BASE_PRECISION_I128);
        assert_eq!(maker_position.quote_asset_amount, 160 * QUOTE_PRECISION);
        assert_eq!(maker_position.quote_entry_amount, 160 * QUOTE_PRECISION);
        assert_eq!(maker_position.unsettled_pnl, 48000);
        assert_eq!(maker_position.open_orders, 0);
        assert_eq!(maker_position.open_asks, 0);
        assert_eq!(maker.fees.total_fee_rebate, 48000);
        assert_eq!(maker.orders[0], Order::default());

        assert_eq!(market.amm.net_base_asset_amount, 0);
        assert_eq!(market.base_asset_amount_long, BASE_PRECISION_I128);
        assert_eq!(market.base_asset_amount_short, -BASE_PRECISION_I128);
        assert_eq!(market.amm.quote_asset_amount_long, 160 * QUOTE_PRECISION);
        assert_eq!(market.amm.quote_asset_amount_short, 160 * QUOTE_PRECISION);
        assert_eq!(market.amm.total_fee, 32000);
        assert_eq!(market.amm.total_fee_minus_distributions, 32000);
        assert_eq!(market.amm.net_revenue_since_last_funding, 32000);
        assert_eq!(market.unsettled_profit, 48000);
        assert_eq!(market.unsettled_loss, 80000);
    }

    #[test]
    fn short_taker_order_fulfilled_start_of_auction() {
        let mut taker = User {
            orders: get_orders(Order {
                market_index: 0,
                order_type: OrderType::Market,
                direction: PositionDirection::Short,
                base_asset_amount: BASE_PRECISION,
                ts: 0,
                slot: 0,
                auction_start_price: 200 * MARK_PRICE_PRECISION,
                auction_end_price: 100 * MARK_PRICE_PRECISION,
                auction_duration: 5,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_asks: -BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            ..User::default()
        };

        let mut maker = User {
            orders: get_orders(Order {
                market_index: 0,
                post_only: true,
                order_type: OrderType::Limit,
                direction: PositionDirection::Long,
                base_asset_amount: BASE_PRECISION,
                ts: 0,
                price: 180 * MARK_PRICE_PRECISION,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_bids: BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            ..User::default()
        };

        let mut market = Market::default();

        let now = 1_i64;
        let slot = 1_u64;

        let fee_structure = get_fee_structure();

        let (taker_key, maker_key, filler_key) = get_user_keys();

        let mut order_records = vec![];

        fulfill_order_with_match(
            &mut market,
            &mut taker,
            0,
            &taker_key,
            &mut maker,
            0,
            &maker_key,
            None,
            &filler_key,
            now,
            slot,
            &fee_structure,
            &mut get_oracle_map(),
            &mut order_records,
        )
        .unwrap();

        let taker_position = &taker.positions[0];
        assert_eq!(taker_position.base_asset_amount, -BASE_PRECISION_I128);
        assert_eq!(taker_position.quote_asset_amount, 180 * QUOTE_PRECISION);
        assert_eq!(taker_position.quote_entry_amount, 180 * QUOTE_PRECISION);
        assert_eq!(taker_position.unsettled_pnl, -90000);
        assert_eq!(taker_position.open_asks, 0);
        assert_eq!(taker_position.open_orders, 0);
        assert_eq!(taker.fees.total_fee_paid, 90000);
        assert_eq!(taker.fees.total_referee_discount, 0);
        assert_eq!(taker.fees.total_token_discount, 0);
        assert_eq!(taker.orders[0], Order::default());

        let maker_position = &maker.positions[0];
        assert_eq!(maker_position.base_asset_amount, BASE_PRECISION_I128);
        assert_eq!(maker_position.quote_asset_amount, 180 * QUOTE_PRECISION);
        assert_eq!(maker_position.quote_entry_amount, 180 * QUOTE_PRECISION);
        assert_eq!(maker_position.unsettled_pnl, 54000);
        assert_eq!(maker_position.open_orders, 0);
        assert_eq!(maker_position.open_bids, 0);
        assert_eq!(maker.fees.total_fee_rebate, 54000);
        assert_eq!(maker.orders[0], Order::default());

        assert_eq!(market.amm.net_base_asset_amount, 0);
        assert_eq!(market.base_asset_amount_long, BASE_PRECISION_I128);
        assert_eq!(market.base_asset_amount_short, -BASE_PRECISION_I128);
        assert_eq!(market.amm.quote_asset_amount_long, 180 * QUOTE_PRECISION);
        assert_eq!(market.amm.quote_asset_amount_short, 180 * QUOTE_PRECISION);
        assert_eq!(market.amm.total_fee, 36000);
        assert_eq!(market.amm.total_fee_minus_distributions, 36000);
        assert_eq!(market.amm.net_revenue_since_last_funding, 36000);
        assert_eq!(market.unsettled_profit, 54000);
        assert_eq!(market.unsettled_loss, 90000);
    }

    #[test]
    fn short_taker_order_fulfilled_middle_of_auction() {
        let mut taker = User {
            orders: get_orders(Order {
                market_index: 0,
                order_type: OrderType::Market,
                direction: PositionDirection::Short,
                base_asset_amount: BASE_PRECISION,
                ts: 0,
                slot: 0,
                auction_start_price: 200 * MARK_PRICE_PRECISION,
                auction_end_price: 100 * MARK_PRICE_PRECISION,
                auction_duration: 5,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_asks: -BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            ..User::default()
        };

        let mut maker = User {
            orders: get_orders(Order {
                market_index: 0,
                post_only: true,
                order_type: OrderType::Limit,
                direction: PositionDirection::Long,
                base_asset_amount: BASE_PRECISION,
                ts: 0,
                price: 140 * MARK_PRICE_PRECISION,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_bids: BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            ..User::default()
        };

        let mut market = Market::default();

        let now = 3_i64;
        let slot = 3_u64;

        let fee_structure = get_fee_structure();

        let (taker_key, maker_key, filler_key) = get_user_keys();

        let mut order_records = vec![];

        fulfill_order_with_match(
            &mut market,
            &mut taker,
            0,
            &taker_key,
            &mut maker,
            0,
            &maker_key,
            None,
            &filler_key,
            now,
            slot,
            &fee_structure,
            &mut get_oracle_map(),
            &mut order_records,
        )
        .unwrap();

        let taker_position = &taker.positions[0];
        assert_eq!(taker_position.base_asset_amount, -BASE_PRECISION_I128);
        assert_eq!(taker_position.quote_asset_amount, 140 * QUOTE_PRECISION);
        assert_eq!(taker_position.quote_entry_amount, 140 * QUOTE_PRECISION);
        assert_eq!(taker_position.unsettled_pnl, -70000);
        assert_eq!(taker_position.open_asks, 0);
        assert_eq!(taker_position.open_orders, 0);
        assert_eq!(taker.fees.total_fee_paid, 70000);
        assert_eq!(taker.fees.total_referee_discount, 0);
        assert_eq!(taker.fees.total_token_discount, 0);
        assert_eq!(taker.orders[0], Order::default());

        let maker_position = &maker.positions[0];
        assert_eq!(maker_position.base_asset_amount, BASE_PRECISION_I128);
        assert_eq!(maker_position.quote_asset_amount, 140 * QUOTE_PRECISION);
        assert_eq!(maker_position.quote_entry_amount, 140 * QUOTE_PRECISION);
        assert_eq!(maker_position.unsettled_pnl, 42000);
        assert_eq!(maker_position.open_orders, 0);
        assert_eq!(maker_position.open_bids, 0);
        assert_eq!(maker.fees.total_fee_rebate, 42000);
        assert_eq!(maker.orders[0], Order::default());

        assert_eq!(market.amm.net_base_asset_amount, 0);
        assert_eq!(market.base_asset_amount_long, BASE_PRECISION_I128);
        assert_eq!(market.base_asset_amount_short, -BASE_PRECISION_I128);
        assert_eq!(market.amm.quote_asset_amount_long, 140 * QUOTE_PRECISION);
        assert_eq!(market.amm.quote_asset_amount_short, 140 * QUOTE_PRECISION);
        assert_eq!(market.amm.total_fee, 28000);
        assert_eq!(market.amm.total_fee_minus_distributions, 28000);
        assert_eq!(market.amm.net_revenue_since_last_funding, 28000);
        assert_eq!(market.unsettled_profit, 42000);
        assert_eq!(market.unsettled_loss, 70000);
    }

    #[test]
    fn long_taker_order_auction_price_does_not_satisfy_maker() {
        let mut taker = User {
            orders: get_orders(Order {
                market_index: 0,
                order_type: OrderType::Market,
                direction: PositionDirection::Long,
                base_asset_amount: BASE_PRECISION,
                ts: 0,
                slot: 0,
                auction_start_price: 100 * MARK_PRICE_PRECISION,
                auction_end_price: 200 * MARK_PRICE_PRECISION,
                auction_duration: 5,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_bids: 100 * BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            ..User::default()
        };

        let mut maker = User {
            orders: get_orders(Order {
                market_index: 0,
                post_only: true,
                order_type: OrderType::Limit,
                direction: PositionDirection::Short,
                base_asset_amount: BASE_PRECISION,
                ts: 0,
                price: 201 * MARK_PRICE_PRECISION,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_bids: 100 * BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            ..User::default()
        };

        let mut market = Market::default();

        let now = 1_i64;
        let slot = 3_u64;

        let fee_structure = FeeStructure::default();

        let (taker_key, maker_key, filler_key) = get_user_keys();

        let mut order_records = vec![];

        let (base_asset_amount, _) = fulfill_order_with_match(
            &mut market,
            &mut taker,
            0,
            &taker_key,
            &mut maker,
            0,
            &maker_key,
            None,
            &filler_key,
            now,
            slot,
            &fee_structure,
            &mut get_oracle_map(),
            &mut order_records,
        )
        .unwrap();

        assert_eq!(base_asset_amount, 0);
    }

    #[test]
    fn short_taker_order_auction_price_does_not_satisfy_maker() {
        let mut taker = User {
            orders: get_orders(Order {
                market_index: 0,
                order_type: OrderType::Market,
                direction: PositionDirection::Short,
                base_asset_amount: BASE_PRECISION,
                ts: 0,
                auction_start_price: 200 * MARK_PRICE_PRECISION,
                auction_end_price: 100 * MARK_PRICE_PRECISION,
                auction_duration: 5,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_bids: 100 * BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            ..User::default()
        };

        let mut maker = User {
            orders: get_orders(Order {
                market_index: 0,
                post_only: true,
                order_type: OrderType::Limit,
                direction: PositionDirection::Long,
                base_asset_amount: BASE_PRECISION,
                ts: 0,
                price: 99 * MARK_PRICE_PRECISION,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_bids: 100 * BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            ..User::default()
        };

        let mut market = Market::default();

        let now = 1_i64;
        let slot = 3_u64;

        let fee_structure = FeeStructure::default();

        let (taker_key, maker_key, filler_key) = get_user_keys();

        let mut order_records = vec![];

        let (base_asset_amount, _) = fulfill_order_with_match(
            &mut market,
            &mut taker,
            0,
            &taker_key,
            &mut maker,
            0,
            &maker_key,
            None,
            &filler_key,
            now,
            slot,
            &fee_structure,
            &mut get_oracle_map(),
            &mut order_records,
        )
        .unwrap();

        assert_eq!(base_asset_amount, 0);
    }

    #[test]
    fn maker_taker_same_direction() {
        let mut taker = User {
            orders: get_orders(Order {
                market_index: 0,
                order_type: OrderType::Market,
                direction: PositionDirection::Short,
                base_asset_amount: BASE_PRECISION,
                ts: 0,
                slot: 0,
                auction_start_price: 200 * MARK_PRICE_PRECISION,
                auction_end_price: 100 * MARK_PRICE_PRECISION,
                auction_duration: 5,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_bids: 100 * BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            ..User::default()
        };

        let mut maker = User {
            orders: get_orders(Order {
                market_index: 0,
                post_only: true,
                order_type: OrderType::Limit,
                direction: PositionDirection::Short,
                base_asset_amount: BASE_PRECISION,
                ts: 0,
                price: 200 * MARK_PRICE_PRECISION,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_bids: 100 * BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            ..User::default()
        };

        let mut market = Market::default();

        let now = 1_i64;
        let slot = 1_u64;

        let fee_structure = FeeStructure::default();

        let (taker_key, maker_key, filler_key) = get_user_keys();

        let mut order_records = vec![];

        let (base_asset_amount, _) = fulfill_order_with_match(
            &mut market,
            &mut taker,
            0,
            &taker_key,
            &mut maker,
            0,
            &maker_key,
            None,
            &filler_key,
            now,
            slot,
            &fee_structure,
            &mut get_oracle_map(),
            &mut order_records,
        )
        .unwrap();

        assert_eq!(base_asset_amount, 0);
    }

    #[test]
    fn maker_taker_different_market_index() {
        let mut taker = User {
            orders: get_orders(Order {
                market_index: 1,
                order_type: OrderType::Market,
                direction: PositionDirection::Short,
                base_asset_amount: BASE_PRECISION,
                ts: 0,
                auction_start_price: 200 * MARK_PRICE_PRECISION,
                auction_end_price: 100 * MARK_PRICE_PRECISION,
                auction_duration: 5,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_bids: 100 * BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            ..User::default()
        };

        let mut maker = User {
            orders: get_orders(Order {
                market_index: 0,
                post_only: true,
                order_type: OrderType::Limit,
                direction: PositionDirection::Long,
                base_asset_amount: BASE_PRECISION,
                ts: 0,
                price: 200 * MARK_PRICE_PRECISION,
                auction_duration: 5,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_bids: 100 * BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            ..User::default()
        };

        let mut market = Market::default();

        let now = 1_i64;
        let slot = 1_u64;

        let fee_structure = FeeStructure::default();

        let (taker_key, maker_key, filler_key) = get_user_keys();

        let mut order_records = vec![];

        let (base_asset_amount, _) = fulfill_order_with_match(
            &mut market,
            &mut taker,
            0,
            &taker_key,
            &mut maker,
            0,
            &maker_key,
            None,
            &filler_key,
            now,
            slot,
            &fee_structure,
            &mut get_oracle_map(),
            &mut order_records,
        )
        .unwrap();

        assert_eq!(base_asset_amount, 0);
    }

    #[test]
    fn long_taker_order_bigger_than_maker() {
        let mut taker = User {
            orders: get_orders(Order {
                market_index: 0,
                order_type: OrderType::Market,
                direction: PositionDirection::Long,
                base_asset_amount: 100 * BASE_PRECISION,
                ts: 0,
                slot: 0,
                auction_start_price: 100 * MARK_PRICE_PRECISION,
                auction_end_price: 200 * MARK_PRICE_PRECISION,
                auction_duration: 5,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_bids: 100 * BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            ..User::default()
        };

        let mut maker = User {
            orders: get_orders(Order {
                market_index: 0,
                post_only: true,
                order_type: OrderType::Limit,
                direction: PositionDirection::Short,
                base_asset_amount: BASE_PRECISION,
                ts: 0,
                price: 120 * MARK_PRICE_PRECISION,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_asks: -BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            ..User::default()
        };

        let mut market = Market::default();

        let now = 1_i64;
        let slot = 1_u64;

        let fee_structure = FeeStructure::default();

        let (taker_key, maker_key, filler_key) = get_user_keys();

        let mut order_records = vec![];

        fulfill_order_with_match(
            &mut market,
            &mut taker,
            0,
            &taker_key,
            &mut maker,
            0,
            &maker_key,
            None,
            &filler_key,
            now,
            slot,
            &fee_structure,
            &mut get_oracle_map(),
            &mut order_records,
        )
        .unwrap();

        let taker_position = &taker.positions[0];
        assert_eq!(taker_position.base_asset_amount, BASE_PRECISION_I128);
        assert_eq!(taker_position.quote_asset_amount, 120 * QUOTE_PRECISION);
        assert_eq!(taker_position.quote_entry_amount, 120 * QUOTE_PRECISION);

        let maker_position = &maker.positions[0];
        assert_eq!(maker_position.base_asset_amount, -BASE_PRECISION_I128);
        assert_eq!(maker_position.quote_asset_amount, 120 * QUOTE_PRECISION);
        assert_eq!(maker_position.quote_entry_amount, 120 * QUOTE_PRECISION);

        assert_eq!(market.amm.net_base_asset_amount, 0);
        assert_eq!(market.base_asset_amount_long, BASE_PRECISION_I128);
        assert_eq!(market.base_asset_amount_short, -BASE_PRECISION_I128);
        assert_eq!(market.amm.quote_asset_amount_long, 120 * QUOTE_PRECISION);
        assert_eq!(market.amm.quote_asset_amount_short, 120 * QUOTE_PRECISION);
    }

    #[test]
    fn long_taker_order_smaller_than_maker() {
        let mut taker = User {
            orders: get_orders(Order {
                market_index: 0,
                order_type: OrderType::Market,
                direction: PositionDirection::Long,
                base_asset_amount: BASE_PRECISION,
                ts: 0,
                slot: 0,
                auction_start_price: 100 * MARK_PRICE_PRECISION,
                auction_end_price: 200 * MARK_PRICE_PRECISION,
                auction_duration: 5,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_bids: 100 * BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            ..User::default()
        };

        let mut maker = User {
            orders: get_orders(Order {
                market_index: 0,
                post_only: true,
                order_type: OrderType::Limit,
                direction: PositionDirection::Short,
                base_asset_amount: 100 * BASE_PRECISION,
                ts: 0,
                price: 120 * MARK_PRICE_PRECISION,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_bids: 100 * BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            ..User::default()
        };

        let mut market = Market::default();

        let now = 1_i64;
        let slot = 1_u64;

        let fee_structure = FeeStructure::default();

        let (taker_key, maker_key, filler_key) = get_user_keys();

        let mut order_records = vec![];

        fulfill_order_with_match(
            &mut market,
            &mut taker,
            0,
            &taker_key,
            &mut maker,
            0,
            &maker_key,
            None,
            &filler_key,
            now,
            slot,
            &fee_structure,
            &mut get_oracle_map(),
            &mut order_records,
        )
        .unwrap();

        let taker_position = &taker.positions[0];
        assert_eq!(taker_position.base_asset_amount, BASE_PRECISION_I128);
        assert_eq!(taker_position.quote_asset_amount, 120 * QUOTE_PRECISION);
        assert_eq!(taker_position.quote_entry_amount, 120 * QUOTE_PRECISION);

        let maker_position = &maker.positions[0];
        assert_eq!(maker_position.base_asset_amount, -BASE_PRECISION_I128);
        assert_eq!(maker_position.quote_asset_amount, 120 * QUOTE_PRECISION);
        assert_eq!(maker_position.quote_entry_amount, 120 * QUOTE_PRECISION);

        assert_eq!(market.amm.net_base_asset_amount, 0);
        assert_eq!(market.base_asset_amount_long, BASE_PRECISION_I128);
        assert_eq!(market.base_asset_amount_short, -BASE_PRECISION_I128);
        assert_eq!(market.amm.quote_asset_amount_long, 120 * QUOTE_PRECISION);
        assert_eq!(market.amm.quote_asset_amount_short, 120 * QUOTE_PRECISION);
    }

    #[test]
    fn double_dutch_auction() {
        let mut taker = User {
            orders: get_orders(Order {
                market_index: 0,
                order_type: OrderType::Market,
                direction: PositionDirection::Long,
                base_asset_amount: BASE_PRECISION,
                ts: 0,
                slot: 0,
                auction_start_price: 100 * MARK_PRICE_PRECISION,
                auction_end_price: 200 * MARK_PRICE_PRECISION,
                auction_duration: 10,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_bids: BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            ..User::default()
        };

        let mut maker = User {
            orders: get_orders(Order {
                market_index: 0,
                post_only: true,
                order_type: OrderType::Market,
                direction: PositionDirection::Short,
                base_asset_amount: BASE_PRECISION,
                ts: 0,
                slot: 0,
                auction_start_price: 200 * MARK_PRICE_PRECISION,
                auction_end_price: 100 * MARK_PRICE_PRECISION,
                auction_duration: 10,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_asks: -BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            ..User::default()
        };

        let mut market = Market::default();

        let now = 5_i64;
        let slot = 5_u64;

        let fee_structure = get_fee_structure();

        let (taker_key, maker_key, filler_key) = get_user_keys();

        let mut order_records = vec![];

        fulfill_order_with_match(
            &mut market,
            &mut taker,
            0,
            &taker_key,
            &mut maker,
            0,
            &maker_key,
            None,
            &filler_key,
            now,
            slot,
            &fee_structure,
            &mut get_oracle_map(),
            &mut order_records,
        )
        .unwrap();

        let taker_position = &taker.positions[0];
        assert_eq!(taker_position.base_asset_amount, BASE_PRECISION_I128);
        assert_eq!(taker_position.quote_asset_amount, 150 * QUOTE_PRECISION);
        assert_eq!(taker_position.quote_entry_amount, 150 * QUOTE_PRECISION);
        assert_eq!(taker_position.unsettled_pnl, -75000);
        assert_eq!(taker_position.open_bids, 0);
        assert_eq!(taker_position.open_orders, 0);
        assert_eq!(taker.fees.total_fee_paid, 75000);
        assert_eq!(taker.fees.total_referee_discount, 0);
        assert_eq!(taker.fees.total_token_discount, 0);
        assert_eq!(taker.orders[0], Order::default());

        let maker_position = &maker.positions[0];
        assert_eq!(maker_position.base_asset_amount, -BASE_PRECISION_I128);
        assert_eq!(maker_position.quote_asset_amount, 150 * QUOTE_PRECISION);
        assert_eq!(maker_position.quote_entry_amount, 150 * QUOTE_PRECISION);
        assert_eq!(maker_position.unsettled_pnl, 45000);
        assert_eq!(maker_position.open_orders, 0);
        assert_eq!(maker_position.open_asks, 0);
        assert_eq!(maker.fees.total_fee_rebate, 45000);
        assert_eq!(maker.orders[0], Order::default());

        assert_eq!(market.amm.net_base_asset_amount, 0);
        assert_eq!(market.base_asset_amount_long, BASE_PRECISION_I128);
        assert_eq!(market.base_asset_amount_short, -BASE_PRECISION_I128);
        assert_eq!(market.amm.quote_asset_amount_long, 150 * QUOTE_PRECISION);
        assert_eq!(market.amm.quote_asset_amount_short, 150 * QUOTE_PRECISION);
        assert_eq!(market.amm.total_fee, 30000);
        assert_eq!(market.amm.total_fee_minus_distributions, 30000);
        assert_eq!(market.amm.net_revenue_since_last_funding, 30000);
        assert_eq!(market.unsettled_profit, 45000);
        assert_eq!(market.unsettled_loss, 75000);
    }

    #[test]
    fn taker_bid_crosses_maker_ask() {
        let mut maker = User {
            orders: get_orders(Order {
                market_index: 0,
                post_only: true,
                order_type: OrderType::Limit,
                direction: PositionDirection::Short,
                base_asset_amount: BASE_PRECISION,
                ts: 0,
                slot: 0,
                price: 100 * MARK_PRICE_PRECISION,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_asks: -BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            ..User::default()
        };

        let mut taker = User {
            orders: get_orders(Order {
                market_index: 0,
                order_type: OrderType::Limit,
                direction: PositionDirection::Long,
                base_asset_amount: BASE_PRECISION,
                ts: 0,
                price: 150 * MARK_PRICE_PRECISION,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_bids: BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            ..User::default()
        };

        let mut market = Market::default();

        let now = 5_i64;
        let slot = 5_u64;

        let fee_structure = get_fee_structure();
        let (maker_key, taker_key, filler_key) = get_user_keys();

        let mut order_records = vec![];

        fulfill_order_with_match(
            &mut market,
            &mut taker,
            0,
            &taker_key,
            &mut maker,
            0,
            &maker_key,
            None,
            &filler_key,
            now,
            slot,
            &fee_structure,
            &mut get_oracle_map(),
            &mut order_records,
        )
        .unwrap();

        let maker_position = &maker.positions[0];
        assert_eq!(maker_position.base_asset_amount, -BASE_PRECISION_I128);
        assert_eq!(maker_position.quote_asset_amount, 100 * QUOTE_PRECISION);
        assert_eq!(maker_position.quote_entry_amount, 100 * QUOTE_PRECISION);
        assert_eq!(maker_position.unsettled_pnl, 30000);
        assert_eq!(maker_position.open_orders, 0);
        assert_eq!(maker_position.open_asks, 0);
        assert_eq!(maker.fees.total_fee_rebate, 30000);
        assert_eq!(maker.orders[0], Order::default());

        let taker_position = &taker.positions[0];
        assert_eq!(taker_position.base_asset_amount, BASE_PRECISION_I128);
        assert_eq!(taker_position.quote_asset_amount, 100 * QUOTE_PRECISION);
        assert_eq!(taker_position.quote_entry_amount, 100 * QUOTE_PRECISION);
        assert_eq!(taker_position.unsettled_pnl, -50000);
        assert_eq!(taker_position.open_bids, 0);
        assert_eq!(taker_position.open_orders, 0);
        assert_eq!(taker.fees.total_fee_paid, 50000);
        assert_eq!(taker.fees.total_referee_discount, 0);
        assert_eq!(taker.fees.total_token_discount, 0);
        assert_eq!(taker.orders[0], Order::default());

        assert_eq!(market.amm.net_base_asset_amount, 0);
        assert_eq!(market.base_asset_amount_long, BASE_PRECISION_I128);
        assert_eq!(market.base_asset_amount_short, -BASE_PRECISION_I128);
        assert_eq!(market.amm.quote_asset_amount_long, 100 * QUOTE_PRECISION);
        assert_eq!(market.amm.quote_asset_amount_short, 100 * QUOTE_PRECISION);
        assert_eq!(market.amm.total_fee, 20000);
        assert_eq!(market.amm.total_fee_minus_distributions, 20000);
        assert_eq!(market.amm.net_revenue_since_last_funding, 20000);
        assert_eq!(market.unsettled_profit, 30000);
        assert_eq!(market.unsettled_loss, 50000);
    }

    #[test]
    fn taker_ask_crosses_maker_bid() {
        let mut maker = User {
            orders: get_orders(Order {
                market_index: 0,
                post_only: true,
                order_type: OrderType::Limit,
                direction: PositionDirection::Long,
                base_asset_amount: BASE_PRECISION,
                ts: 0,
                slot: 0,
                price: 100 * MARK_PRICE_PRECISION,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_bids: BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            ..User::default()
        };

        let mut taker = User {
            orders: get_orders(Order {
                market_index: 0,
                order_type: OrderType::Limit,
                direction: PositionDirection::Short,
                base_asset_amount: BASE_PRECISION,
                ts: 0,
                price: 50 * MARK_PRICE_PRECISION,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_asks: -BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            ..User::default()
        };

        let mut market = Market::default();

        let now = 5_i64;
        let slot = 5_u64;

        let fee_structure = get_fee_structure();

        let (maker_key, taker_key, filler_key) = get_user_keys();

        let mut order_records = vec![];

        fulfill_order_with_match(
            &mut market,
            &mut taker,
            0,
            &taker_key,
            &mut maker,
            0,
            &maker_key,
            None,
            &filler_key,
            now,
            slot,
            &fee_structure,
            &mut get_oracle_map(),
            &mut order_records,
        )
        .unwrap();

        let maker_position = &maker.positions[0];
        assert_eq!(maker_position.base_asset_amount, BASE_PRECISION_I128);
        assert_eq!(maker_position.quote_asset_amount, 100 * QUOTE_PRECISION);
        assert_eq!(maker_position.quote_entry_amount, 100 * QUOTE_PRECISION);
        assert_eq!(maker_position.unsettled_pnl, 30000);
        assert_eq!(maker_position.open_orders, 0);
        assert_eq!(maker_position.open_bids, 0);
        assert_eq!(maker.fees.total_fee_rebate, 30000);
        assert_eq!(maker.orders[0], Order::default());

        let taker_position = &taker.positions[0];
        assert_eq!(taker_position.base_asset_amount, -BASE_PRECISION_I128);
        assert_eq!(taker_position.quote_asset_amount, 100 * QUOTE_PRECISION);
        assert_eq!(taker_position.quote_entry_amount, 100 * QUOTE_PRECISION);
        assert_eq!(taker_position.unsettled_pnl, -50000);
        assert_eq!(taker_position.open_asks, 0);
        assert_eq!(taker_position.open_orders, 0);
        assert_eq!(taker.fees.total_fee_paid, 50000);
        assert_eq!(taker.fees.total_referee_discount, 0);
        assert_eq!(taker.fees.total_token_discount, 0);
        assert_eq!(taker.orders[0], Order::default());

        assert_eq!(market.amm.net_base_asset_amount, 0);
        assert_eq!(market.base_asset_amount_long, BASE_PRECISION_I128);
        assert_eq!(market.base_asset_amount_short, -BASE_PRECISION_I128);
        assert_eq!(market.amm.quote_asset_amount_long, 100 * QUOTE_PRECISION);
        assert_eq!(market.amm.quote_asset_amount_short, 100 * QUOTE_PRECISION);
        assert_eq!(market.amm.total_fee, 20000);
        assert_eq!(market.amm.total_fee_minus_distributions, 20000);
        assert_eq!(market.amm.net_revenue_since_last_funding, 20000);
        assert_eq!(market.unsettled_profit, 30000);
        assert_eq!(market.unsettled_loss, 50000);
    }
}

pub mod fulfill_order {
    use super::*;
    use crate::controller::orders::fulfill_order;
    use crate::controller::position::PositionDirection;
    use crate::create_account_info;
    use crate::math::constants::{
        AMM_RESERVE_PRECISION, BANK_CUMULATIVE_INTEREST_PRECISION, BANK_INTEREST_PRECISION,
        BANK_WEIGHT_PRECISION, BASE_PRECISION, BASE_PRECISION_I128, MARK_PRICE_PRECISION,
        PEG_PRECISION, QUOTE_PRECISION,
    };
    use crate::state::bank::{Bank, BankBalanceType};
    use crate::state::bank_map::BankMap;
    use crate::state::market::{Market, AMM};
    use crate::state::market_map::MarketMap;
    use crate::state::oracle::OracleSource;
    use crate::state::user::{OrderStatus, OrderType, User, UserBankBalance};
    use crate::tests::utils::*;
    use std::ops::Deref;

    #[test]
    fn fulfill_with_amm_and_maker() {
        let mut market = Market {
            amm: AMM {
                base_asset_reserve: 100 * AMM_RESERVE_PRECISION,
                quote_asset_reserve: 100 * AMM_RESERVE_PRECISION,
                bid_base_asset_reserve: 101 * AMM_RESERVE_PRECISION,
                bid_quote_asset_reserve: 99 * AMM_RESERVE_PRECISION,
                ask_base_asset_reserve: 99 * AMM_RESERVE_PRECISION,
                ask_quote_asset_reserve: 101 * AMM_RESERVE_PRECISION,
                sqrt_k: 100 * AMM_RESERVE_PRECISION,
                peg_multiplier: 100 * PEG_PRECISION,
                ..AMM::default()
            },
            margin_ratio_initial: 1000,
            margin_ratio_partial: 714,
            margin_ratio_maintenance: 500,
            initialized: true,
            ..Market::default()
        };
        create_account_info!(market, Market, market_account_info);
        let market_map = MarketMap::load_one(&market_account_info, true).unwrap();

        let mut bank = Bank {
            bank_index: 0,
            oracle_source: OracleSource::QuoteAsset,
            cumulative_deposit_interest: BANK_CUMULATIVE_INTEREST_PRECISION,
            decimals: 6,
            initial_asset_weight: BANK_WEIGHT_PRECISION,
            ..Bank::default()
        };
        create_account_info!(bank, Bank, bank_account_info);
        let bank_map = BankMap::load_one(&bank_account_info, true).unwrap();

        let mut oracle_map = get_oracle_map();

        let mut taker = User {
            orders: get_orders(Order {
                market_index: 0,
                status: OrderStatus::Open,
                order_type: OrderType::Market,
                direction: PositionDirection::Long,
                base_asset_amount: BASE_PRECISION,
                ts: 0,
                slot: 0,
                auction_start_price: 0,
                auction_end_price: 100 * MARK_PRICE_PRECISION,
                auction_duration: 0,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_bids: BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            bank_balances: get_bank_balances(UserBankBalance {
                bank_index: 0,
                balance_type: BankBalanceType::Deposit,
                balance: 100 * BANK_INTEREST_PRECISION,
            }),
            ..User::default()
        };

        let mut maker = User {
            orders: get_orders(Order {
                market_index: 0,
                post_only: true,
                order_type: OrderType::Limit,
                direction: PositionDirection::Short,
                base_asset_amount: BASE_PRECISION / 2,
                ts: 0,
                price: 100 * MARK_PRICE_PRECISION,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_asks: -BASE_PRECISION_I128 / 2,
                ..MarketPosition::default()
            }),
            ..User::default()
        };

        let now = 0_i64;
        let slot = 0_u64;

        let fee_structure = get_fee_structure();

        let (taker_key, maker_key, filler_key) = get_user_keys();

        let (base_asset_amount, _) = fulfill_order(
            &mut taker,
            0,
            &taker_key,
            &mut Some(&mut maker),
            Some(0),
            Some(&maker_key),
            &mut None,
            &filler_key,
            &bank_map,
            &market_map,
            &mut oracle_map,
            &fee_structure,
            0,
            None,
            now,
            slot,
        )
        .unwrap();

        assert_eq!(base_asset_amount, BASE_PRECISION);

        let taker_position = &taker.positions[0];
        assert_eq!(taker_position.base_asset_amount, BASE_PRECISION_I128);
        assert_eq!(taker_position.quote_asset_amount, 102284264);
        assert_eq!(taker_position.quote_entry_amount, 102284264);
        assert_eq!(taker_position.unsettled_pnl, -51142);
        assert_eq!(taker_position.open_bids, 0);
        assert_eq!(taker_position.open_orders, 0);
        assert_eq!(taker.fees.total_fee_paid, 51142);
        assert_eq!(taker.fees.total_referee_discount, 0);
        assert_eq!(taker.fees.total_token_discount, 0);
        assert_eq!(taker.orders[0], Order::default());

        let maker_position = &maker.positions[0];
        assert_eq!(maker_position.base_asset_amount, -BASE_PRECISION_I128 / 2);
        assert_eq!(maker_position.quote_asset_amount, 50 * QUOTE_PRECISION);
        assert_eq!(maker_position.quote_entry_amount, 50 * QUOTE_PRECISION);
        assert_eq!(maker_position.unsettled_pnl, 15000);
        assert_eq!(maker_position.open_orders, 0);
        assert_eq!(maker_position.open_asks, 0);
        assert_eq!(maker.fees.total_fee_rebate, 15000);
        assert_eq!(maker.orders[0], Order::default());

        let market_after = market_map.get_ref(&0).unwrap();
        assert_eq!(market_after.amm.net_base_asset_amount, 5000000000000);
        assert_eq!(market_after.base_asset_amount_long, 10000000000000);
        assert_eq!(market_after.base_asset_amount_short, -5000000000000);
        assert_eq!(market_after.amm.quote_asset_amount_long, 102284264);
        assert_eq!(market_after.amm.quote_asset_amount_short, 50000000);
        assert_eq!(market_after.amm.total_fee, 2069149);
        assert_eq!(market_after.amm.total_fee_minus_distributions, 2069149);
        assert_eq!(market_after.amm.net_revenue_since_last_funding, 2069149);
        assert_eq!(market_after.unsettled_profit, 15000);
        assert_eq!(market_after.unsettled_loss, 51142);
    }

    #[test]
    fn fulfill_with_maker_with_auction_incomplete() {
        let mut market = Market {
            amm: AMM {
                base_asset_reserve: 100 * AMM_RESERVE_PRECISION,
                quote_asset_reserve: 100 * AMM_RESERVE_PRECISION,
                bid_base_asset_reserve: 101 * AMM_RESERVE_PRECISION,
                bid_quote_asset_reserve: 99 * AMM_RESERVE_PRECISION,
                ask_base_asset_reserve: 99 * AMM_RESERVE_PRECISION,
                ask_quote_asset_reserve: 101 * AMM_RESERVE_PRECISION,
                sqrt_k: 100 * AMM_RESERVE_PRECISION,
                peg_multiplier: 100 * PEG_PRECISION,
                ..AMM::default()
            },
            margin_ratio_initial: 1000,
            margin_ratio_partial: 714,
            margin_ratio_maintenance: 500,
            initialized: true,
            ..Market::default()
        };
        create_account_info!(market, Market, market_account_info);
        let market_map = MarketMap::load_one(&market_account_info, true).unwrap();

        let mut bank = Bank {
            bank_index: 0,
            oracle_source: OracleSource::QuoteAsset,
            cumulative_deposit_interest: BANK_CUMULATIVE_INTEREST_PRECISION,
            decimals: 6,
            initial_asset_weight: BANK_WEIGHT_PRECISION,
            ..Bank::default()
        };
        create_account_info!(bank, Bank, bank_account_info);
        let bank_map = BankMap::load_one(&bank_account_info, true).unwrap();

        let mut oracle_map = get_oracle_map();

        let mut taker = User {
            orders: get_orders(Order {
                market_index: 0,
                status: OrderStatus::Open,
                order_type: OrderType::Market,
                direction: PositionDirection::Long,
                base_asset_amount: BASE_PRECISION,
                ts: 0,
                slot: 0,
                auction_start_price: 100 * MARK_PRICE_PRECISION,
                auction_end_price: 200 * MARK_PRICE_PRECISION,
                auction_duration: 5,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_bids: BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            bank_balances: get_bank_balances(UserBankBalance {
                bank_index: 0,
                balance_type: BankBalanceType::Deposit,
                balance: 100 * BANK_INTEREST_PRECISION,
            }),
            ..User::default()
        };

        let mut maker = User {
            orders: get_orders(Order {
                market_index: 0,
                post_only: true,
                order_type: OrderType::Limit,
                direction: PositionDirection::Short,
                base_asset_amount: BASE_PRECISION / 2,
                ts: 0,
                price: 100 * MARK_PRICE_PRECISION,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_asks: -BASE_PRECISION_I128 / 2,
                ..MarketPosition::default()
            }),
            ..User::default()
        };

        let now = 0_i64;
        let slot = 0_u64;

        let fee_structure = get_fee_structure();

        let (taker_key, maker_key, filler_key) = get_user_keys();

        let (base_asset_amount, _) = fulfill_order(
            &mut taker,
            0,
            &taker_key,
            &mut Some(&mut maker),
            Some(0),
            Some(&maker_key),
            &mut None,
            &filler_key,
            &bank_map,
            &market_map,
            &mut oracle_map,
            &fee_structure,
            0,
            None,
            now,
            slot,
        )
        .unwrap();

        assert_eq!(base_asset_amount, BASE_PRECISION / 2);

        let taker_position = &taker.positions[0];
        assert_eq!(taker_position.base_asset_amount, BASE_PRECISION_I128 / 2);
        assert_eq!(taker_position.quote_asset_amount, 50 * QUOTE_PRECISION);
        assert_eq!(taker_position.quote_entry_amount, 50 * QUOTE_PRECISION);
        assert_eq!(taker_position.unsettled_pnl, -25000);
        assert_eq!(taker_position.open_bids, BASE_PRECISION_I128 / 2);
        assert_eq!(taker_position.open_orders, 1);
        assert_eq!(taker.fees.total_fee_paid, 25000);
        assert_eq!(taker.fees.total_referee_discount, 0);
        assert_eq!(taker.fees.total_token_discount, 0);

        let maker_position = &maker.positions[0];
        assert_eq!(maker_position.base_asset_amount, -BASE_PRECISION_I128 / 2);
        assert_eq!(maker_position.quote_asset_amount, 50 * QUOTE_PRECISION);
        assert_eq!(maker_position.quote_entry_amount, 50 * QUOTE_PRECISION);
        assert_eq!(maker_position.unsettled_pnl, 15000);
        assert_eq!(maker_position.open_orders, 0);
        assert_eq!(maker_position.open_asks, 0);
        assert_eq!(maker.fees.total_fee_rebate, 15000);

        let market_after = market_map.get_ref(&0).unwrap();
        assert_eq!(market_after.amm.net_base_asset_amount, 0);
        assert_eq!(market_after.base_asset_amount_long, 5000000000000);
        assert_eq!(market_after.base_asset_amount_short, -5000000000000);
        assert_eq!(market_after.amm.quote_asset_amount_long, 50000000);
        assert_eq!(market_after.amm.quote_asset_amount_short, 50000000);
        assert_eq!(market_after.amm.total_fee, 10000);
        assert_eq!(market_after.amm.total_fee_minus_distributions, 10000);
        assert_eq!(market_after.amm.net_revenue_since_last_funding, 10000);
        assert_eq!(market_after.unsettled_profit, 15000);
        assert_eq!(market_after.unsettled_loss, 25000);
    }

    #[test]
    fn fulfill_with_amm_end_of_auction() {
        let mut market = Market {
            amm: AMM {
                base_asset_reserve: 100 * AMM_RESERVE_PRECISION,
                quote_asset_reserve: 100 * AMM_RESERVE_PRECISION,
                bid_base_asset_reserve: 101 * AMM_RESERVE_PRECISION,
                bid_quote_asset_reserve: 99 * AMM_RESERVE_PRECISION,
                ask_base_asset_reserve: 99 * AMM_RESERVE_PRECISION,
                ask_quote_asset_reserve: 101 * AMM_RESERVE_PRECISION,
                sqrt_k: 100 * AMM_RESERVE_PRECISION,
                peg_multiplier: 100 * PEG_PRECISION,
                ..AMM::default()
            },
            margin_ratio_initial: 1000,
            margin_ratio_partial: 714,
            margin_ratio_maintenance: 500,
            initialized: true,
            ..Market::default()
        };
        create_account_info!(market, Market, market_account_info);
        let market_map = MarketMap::load_one(&market_account_info, true).unwrap();

        let mut bank = Bank {
            bank_index: 0,
            oracle_source: OracleSource::QuoteAsset,
            cumulative_deposit_interest: BANK_CUMULATIVE_INTEREST_PRECISION,
            decimals: 6,
            initial_asset_weight: BANK_WEIGHT_PRECISION,
            ..Bank::default()
        };
        create_account_info!(bank, Bank, bank_account_info);
        let bank_map = BankMap::load_one(&bank_account_info, true).unwrap();

        let mut oracle_map = get_oracle_map();

        let mut taker = User {
            orders: get_orders(Order {
                market_index: 0,
                status: OrderStatus::Open,
                order_type: OrderType::Market,
                direction: PositionDirection::Long,
                base_asset_amount: BASE_PRECISION,
                ts: 0,
                slot: 0,
                auction_start_price: 0,
                auction_end_price: 100 * MARK_PRICE_PRECISION,
                auction_duration: 5,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_bids: BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            bank_balances: get_bank_balances(UserBankBalance {
                bank_index: 0,
                balance_type: BankBalanceType::Deposit,
                balance: 100 * BANK_INTEREST_PRECISION,
            }),
            ..User::default()
        };

        let now = 0_i64;
        let slot = 5_u64;

        let fee_structure = get_fee_structure();

        let (taker_key, _, filler_key) = get_user_keys();

        let (base_asset_amount, _) = fulfill_order(
            &mut taker,
            0,
            &taker_key,
            &mut None,
            None,
            None,
            &mut None,
            &filler_key,
            &bank_map,
            &market_map,
            &mut oracle_map,
            &fee_structure,
            0,
            None,
            now,
            slot,
        )
        .unwrap();

        assert_eq!(base_asset_amount, BASE_PRECISION);

        let taker_position = &taker.positions[0];
        assert_eq!(taker_position.base_asset_amount, BASE_PRECISION_I128);
        assert_eq!(taker_position.quote_asset_amount, 104081633);
        assert_eq!(taker_position.quote_entry_amount, 104081633);
        assert_eq!(taker_position.unsettled_pnl, -52040);
        assert_eq!(taker_position.open_bids, 0);
        assert_eq!(taker_position.open_orders, 0);
        assert_eq!(taker.fees.total_fee_paid, 52040);
        assert_eq!(taker.fees.total_referee_discount, 0);
        assert_eq!(taker.fees.total_token_discount, 0);
        assert_eq!(taker.orders[0], Order::default());

        let market_after = market_map.get_ref(&0).unwrap();
        assert_eq!(market_after.amm.net_base_asset_amount, 10000000000000);
        assert_eq!(market_after.base_asset_amount_long, 10000000000000);
        assert_eq!(market_after.base_asset_amount_short, 0);
        assert_eq!(market_after.amm.quote_asset_amount_long, 104081633);
        assert_eq!(market_after.amm.quote_asset_amount_short, 0);
        assert_eq!(market_after.amm.total_fee, 3123571);
        assert_eq!(market_after.amm.total_fee_minus_distributions, 3123571);
        assert_eq!(market_after.amm.net_revenue_since_last_funding, 3123571);
        assert_eq!(market_after.unsettled_profit, 0);
        assert_eq!(market_after.unsettled_loss, 52040);
    }

    #[test]
    fn taker_breaches_margin_requirement() {
        let mut market = Market {
            amm: AMM {
                base_asset_reserve: 100 * AMM_RESERVE_PRECISION,
                quote_asset_reserve: 100 * AMM_RESERVE_PRECISION,
                bid_base_asset_reserve: 101 * AMM_RESERVE_PRECISION,
                bid_quote_asset_reserve: 99 * AMM_RESERVE_PRECISION,
                ask_base_asset_reserve: 99 * AMM_RESERVE_PRECISION,
                ask_quote_asset_reserve: 101 * AMM_RESERVE_PRECISION,
                sqrt_k: 100 * AMM_RESERVE_PRECISION,
                peg_multiplier: 100 * PEG_PRECISION,
                ..AMM::default()
            },
            margin_ratio_initial: 1000,
            margin_ratio_partial: 714,
            margin_ratio_maintenance: 500,
            initialized: true,
            ..Market::default()
        };
        create_account_info!(market, Market, market_account_info);
        let market_map = MarketMap::load_one(&market_account_info, true).unwrap();

        let mut bank = Bank {
            bank_index: 0,
            oracle_source: OracleSource::QuoteAsset,
            cumulative_deposit_interest: BANK_CUMULATIVE_INTEREST_PRECISION,
            decimals: 6,
            initial_asset_weight: BANK_WEIGHT_PRECISION,
            ..Bank::default()
        };
        create_account_info!(bank, Bank, bank_account_info);
        let bank_map = BankMap::load_one(&bank_account_info, true).unwrap();

        let mut oracle_map = get_oracle_map();

        let mut taker = User {
            orders: get_orders(Order {
                market_index: 0,
                status: OrderStatus::Open,
                order_type: OrderType::Market,
                direction: PositionDirection::Long,
                base_asset_amount: BASE_PRECISION,
                ts: 0,
                slot: 0,
                auction_start_price: 0,
                auction_end_price: 100 * MARK_PRICE_PRECISION,
                auction_duration: 0,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_bids: BASE_PRECISION_I128,
                ..MarketPosition::default()
            }),
            bank_balances: get_bank_balances(UserBankBalance {
                bank_index: 0,
                balance_type: BankBalanceType::Deposit,
                balance: BANK_INTEREST_PRECISION,
            }),
            ..User::default()
        };

        let mut maker = User {
            orders: get_orders(Order {
                market_index: 0,
                post_only: true,
                order_type: OrderType::Limit,
                direction: PositionDirection::Short,
                base_asset_amount: BASE_PRECISION / 2,
                ts: 0,
                price: 100 * MARK_PRICE_PRECISION,
                ..Order::default()
            }),
            positions: get_positions(MarketPosition {
                market_index: 0,
                open_orders: 1,
                open_asks: -BASE_PRECISION_I128 / 2,
                ..MarketPosition::default()
            }),
            ..User::default()
        };

        let mut filler = User::default();

        let now = 0_i64;
        let slot = 0_u64;

        let fee_structure = get_fee_structure();

        let (taker_key, maker_key, filler_key) = get_user_keys();

        let expected_taker_after = User {
            positions: get_positions(MarketPosition {
                market_index: 0,
                ..MarketPosition::default()
            }),
            orders: get_orders(Order::default()),
            ..taker
        };
        let expected_maker_after = maker;
        let expected_filler_after = filler;
        let expected_market_after = *market_map.get_ref(&0).unwrap();

        let (base_asset_amount, potentially_risk_increasing) = fulfill_order(
            &mut taker,
            0,
            &taker_key,
            &mut Some(&mut maker),
            Some(0),
            Some(&maker_key),
            &mut Some(&mut filler),
            &filler_key,
            &bank_map,
            &market_map,
            &mut oracle_map,
            &fee_structure,
            0,
            None,
            now,
            slot,
        )
        .unwrap();

        assert_eq!(base_asset_amount, 0);
        assert!(!potentially_risk_increasing);
        assert_eq!(maker, expected_maker_after);
        assert_eq!(taker, expected_taker_after);
        assert_eq!(filler, expected_filler_after);

        let market_after = market_map.get_ref(&0).unwrap();
        assert_eq!(*market_after.deref(), expected_market_after);
    }
}
export const types = {
  Side: {
    _enum: ['Buy', 'Sell'],
  },
  Direction: {
    _enum: ['Long', 'Short'],
  },
  Position: {
    market_id: 'u128',
    direction: 'Direction',
    side: 'Side',
    avg_execution_price: 'FixedI128',
    size: 'FixedI128',
    margin_amount: 'FixedI128',
    borrowed_amount: 'FixedI128',
    leverage: 'FixedI128',
    realized_pnl: 'FixedI128',
  },
};

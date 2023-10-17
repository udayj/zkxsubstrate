export const rpc = {
  trading: {
    get_positions: {
      description: 'Just a test method',
      type: 'Vec<Position>',
      params: [
        {
          name: 'account_id',
          type: 'u256',
        },
        {
          name: 'collateral_id',
          type: 'u128',
        },
        {
          name: 'at',
          type: 'Hash',
          isOptional: true,
        },
      ],
    },
  },
};

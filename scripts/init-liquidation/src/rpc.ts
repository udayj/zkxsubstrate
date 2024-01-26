export const rpc = {
  abr: {
    get_last_timestamp: {
      description: "Returns the last timestamp when ABR was performed",
      type: "u64",
      params: [
        {
          name: "at",
          type: "Hash",
          isOptional: true,
        },
      ],
    },
    get_next_timestamp: {
      description: "Returns the next ABR execution timestamp",
      type: "u64",
      params: [
        {
          name: "at",
          type: "Hash",
          isOptional: true,
        },
      ],
    },
  },
};

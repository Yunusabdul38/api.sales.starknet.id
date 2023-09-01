import {
  uint256,
  formatUnits,
  Block,
  EventWithTransaction,
} from "./common/deps.ts";
import {
  formatFelt,
  TAX_CONTRACT,
  SELECTOR_KEYS,
  DECIMALS,
  DB_NAME,
  MONGO_CONNECTION_STRING,
} from "./common/constants.ts";

const filter = {
  header: { weak: true },
  events: [
    {
      fromAddress: Deno.env.get("ETH_CONTRACT"),
      keys: [formatFelt(SELECTOR_KEYS.TRANSFER)],
    },
  ],
};

export const config = {
  streamUrl: Deno.env.get("STREAM_URL"),
  startingBlock: Number(Deno.env.get("STARTING_BLOCK")),
  network: "starknet",
  filter,
  sinkType: "mongo",
  sinkOptions: {
    connectionString: MONGO_CONNECTION_STRING,
    database: DB_NAME,
    collectionName: "tax_txs",
    entityMode: false,
  },
};

type TaxTxDocument = {
  tx_hash: string;
  amount_eth: number;
};

export default function transform({ events }: Block) {
  // Mapping and decoding each event in the block
  const decodedEvents = events.map(
    ({ event, transaction }: EventWithTransaction) => {
      const [_, toAddress, amountLow, amountHigh] = event.data;
      if (BigInt(toAddress) !== TAX_CONTRACT) return;

      return {
        tx_hash: transaction.meta.hash,
        amount_eth: +formatUnits(
          uint256.uint256ToBN({ low: amountLow, high: amountHigh }),
          DECIMALS
        ),
      };
    }
  );

  // Filtering out undefined or null values from the decoded events array
  return decodedEvents.filter(Boolean) as TaxTxDocument[];
}

import { uint256, Block, EventWithTransaction } from "./common/deps.ts";
import {
  formatFelt,
  SELECTOR_KEYS,
  DB_NAME,
  MONGO_CONNECTION_STRING,
  FINALITY,
} from "./common/constants.ts";
import { decodeRootDomain } from "./common/starknetid.ts";

const filter = {
  header: { weak: true },
  events: [
    {
      fromAddress: Deno.env.get("RENEWAL_CONTRACT"),
      keys: [formatFelt(SELECTOR_KEYS.UPDATE_AUTO_RENEW)],
      includeTransaction: true,
      includeReceipt: false,
    },
    {
      fromAddress: Deno.env.get("RENEWAL_CONTRACT"),
      keys: [formatFelt(SELECTOR_KEYS.DISABLE_AUTO_RENEW)],
      includeTransaction: true,
      includeReceipt: false,
    },
  ],
};

export const config = {
  streamUrl: Deno.env.get("STREAM_URL"),
  startingBlock: Number(Deno.env.get("STARTING_BLOCK")),
  network: "starknet",
  filter,
  sinkType: "mongo",
  finality: FINALITY,
  sinkOptions: {
    connectionString: MONGO_CONNECTION_STRING,
    database: DB_NAME,
    collectionName: "auto_renew_updates",
    entityMode: true,
  },
};

export default function transform({ events }: Block) {
  // Mapping and decoding each event in the block
  const decodedEvents = events.map(
    ({ event, transaction }: EventWithTransaction) => {
      const key = BigInt(event.keys[0]);

      switch (key) {
        case SELECTOR_KEYS.UPDATE_AUTO_RENEW: {
          const [_, _domain] = event.keys;
          const [renewer, amountLow, amountHigh, metaHash] = event.data;
          const domain = decodeRootDomain(BigInt(_domain));
          return {
            entity: {
              domain,
              renewer,
            },
            update: [
              {
                $set: {
                  domain,
                  renewer,
                  allowance: uint256
                    .uint256ToBN({ low: amountLow, high: amountHigh })
                    .toString(),
                  meta_hash: metaHash.slice(4),
                  tx_hash: transaction.meta.hash,
                },
              },
            ],
          };
        }

        case SELECTOR_KEYS.DISABLE_AUTO_RENEW: {
          const [_, _domain] = event.keys;
          const [renewer] = event.data;
          const domain = decodeRootDomain(BigInt(_domain));
          return {
            entity: {
              domain,
              renewer,
            },
            update: [
              {
                $set: {
                  domain,
                  renewer,
                  allowance: "0",
                  tx_hash: transaction.meta.hash,
                },
              },
            ],
          };
        }

        default:
          return;
      }
    }
  );

  // Filtering out undefined or null values from the decoded events array
  return decodedEvents.filter(Boolean);
}

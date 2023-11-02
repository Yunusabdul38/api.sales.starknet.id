import { hash } from "./deps.ts";

export function formatFelt(key: bigint): string {
  return "0x" + key.toString(16);
}

export const SELECTOR_KEYS = {
  TRANSFER: BigInt(hash.getSelectorFromName("Transfer")),
  STARK_UPDATE: BigInt(hash.getSelectorFromName("starknet_id_update")),
  SALE_METADATA: BigInt(hash.getSelectorFromName("SaleMetadata")),
  AUTO_RENEW: BigInt(hash.getSelectorFromName("domain_renewed")),
  UPDATE_AUTO_RENEW : BigInt(hash.getSelectorFromName("UpdatedRenewal")),
  DISABLE_AUTO_RENEW : BigInt(hash.getSelectorFromName("DisabledRenewal")),

  REFERRAL: BigInt(hash.getSelectorFromName("on_commission")),
};

export const FINALITY = Deno.env.get("FINALITY") as string;
export const MONGO_CONNECTION_STRING = Deno.env.get(
  "MONGO_CONNECTION_STRING"
) as string;
export const DB_NAME = Deno.env.get("DB_NAME") as string;
export const NAMING_CONTRACT = BigInt(
  Deno.env.get("NAMING_CONTRACT") as string
);
export const RENEWAL_CONTRACT = BigInt(
  Deno.env.get("RENEWAL_CONTRACT") as string
);
export const TAX_CONTRACT = BigInt(Deno.env.get("TAX_CONTRACT") as string);
export const DECIMALS = 18;

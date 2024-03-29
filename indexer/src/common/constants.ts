import { hash } from "./deps.ts";

export function formatFelt(key: bigint): string {
  return "0x" + key.toString(16);
}

export const SELECTOR_KEYS = {
  TRANSFER: BigInt(hash.getSelectorFromName("Transfer")),
  DOMAIN_UPDATE: BigInt(hash.getSelectorFromName("starknet_id_update")),
  SALE_METADATA: BigInt(hash.getSelectorFromName("SaleMetadata")),
  DOMAIN_MINT: BigInt(hash.getSelectorFromName("DomainMint")),
  AUTO_RENEW: BigInt(hash.getSelectorFromName("domain_renewed")),
  UPDATE_AUTO_RENEW: BigInt(hash.getSelectorFromName("UpdatedRenewal")),
  DISABLE_AUTO_RENEW: BigInt(hash.getSelectorFromName("DisabledRenewal")),

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
export const NAMING_UPGRADE_A_BLOCK = Number(
  Deno.env.get("NAMING_UPGRADE_A_BLOCK")
);
export const TAX_CONTRACT = BigInt(Deno.env.get("TAX_CONTRACT") as string);
export const DECIMALS = 18;

const TOKEN_CONTRACTS_LEN = parseInt(
  Deno.env.get("TOKEN_CONTRACTS_LEN") as string
);

// Dynamically retrieve each token contract
export const TOKEN_CONTRACTS_STRINGS: string[] = [];

for (let i = 0; i < TOKEN_CONTRACTS_LEN; i++) {
  const tokenContractEnvName = `TOKEN_CONTRACT_${i}`;
  const tokenContract = Deno.env.get(tokenContractEnvName) as string;
  TOKEN_CONTRACTS_STRINGS.push(tokenContract);
}

import fs from "fs";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import {
  publicKey,
  none,
  createSignerFromKeypair,
  signerIdentity,
} from "@metaplex-foundation/umi";
import {
  mplTokenMetadata,
  createMetadataAccountV3,
} from "@metaplex-foundation/mpl-token-metadata";

// ENV inputs
const RPC = process.env.RPC || "https://api.devnet.solana.com";
const MINT = process.env.MINT || "HbA6BgBmA3X6X8jtts5X2ZiJXXxZQKDbQR4s5XCD82pr";
const NAME = process.env.NAME ?? "BONK";
const SYMBOL = process.env.SYMBOL ?? "BONK";
const URI =
  process.env.URI ??
  "https://gist.githubusercontent.com/reymom/2d200a77435294a765622b84473284d3/raw/61af2351882247e63303161c694ff4d576058fce/bonk.json";
const KEY = process.env.KEYPAIR || `${process.env.HOME}/.config/solana/id.json`;

if (!MINT) throw new Error("MINT env var required");
const umi = createUmi(RPC).use(mplTokenMetadata());

const secret = JSON.parse(fs.readFileSync(KEY, "utf8"));
const kp = umi.eddsa.createKeypairFromSecretKey(Uint8Array.from(secret));

const signer = createSignerFromKeypair(umi, kp);
umi.use(signerIdentity(signer));

const res = await createMetadataAccountV3(umi, {
  // Accounts
  mint: publicKey(MINT),
  mintAuthority: signer,
  payer: signer,
  updateAuthority: signer,
  // Data
  data: {
    name: NAME,
    symbol: SYMBOL,
    uri: URI,
    sellerFeeBasisPoints: 0,
    creators: none(),
    collection: none(),
    uses: none(),
  },
  isMutable: true,
  collectionDetails: none(),
}).sendAndConfirm(umi);

console.log("OK metadata tx:", res.signature);
console.log(`symbol=${SYMBOL} uri=${URI}`);

// node scripts/set_token_metadata.mjs

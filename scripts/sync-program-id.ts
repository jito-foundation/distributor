import fs from "fs";
import path from "path";

const KEYPAIR_PATH = path.join("target", "deploy", "merkle_distributor-keypair.json");
const LIB_RS = path.join("programs", "merkle-distributor", "src", "lib.rs");
const ANCHOR_TOML = "Anchor.toml";
const IDL_JSON = path.join("target", "idl", "merkle_distributor.json");

function readProgramIdFromKeypair(jsonPath: string): string {
  const raw = fs.readFileSync(jsonPath, "utf-8");
  const key = JSON.parse(raw);
  const { Keypair } = require("@solana/web3.js");
  const kp = Keypair.fromSecretKey(Uint8Array.from(key));
  return kp.publicKey.toBase58();
}

function replaceDeclareId(file: string, programId: string) {
  let src = fs.readFileSync(file, "utf-8");
  src = src.replace(/declare_id!\("?[A-Za-z0-9]{32,44}"?\);/, `declare_id!("${programId}");`);
  fs.writeFileSync(file, src);
}

function replaceAnchorTomlProgramId(file: string, programId: string) {
  let t = fs.readFileSync(file, "utf-8");
  if (/\[programs\.localnet\]/.test(t)) {
    if (/merkle_distributor\s*=/.test(t)) {
      t = t.replace(/(merkle_distributor\s*=\s*")([A-Za-z0-9]{32,44})(")/, `$1${programId}$3`);
    } else {
      t = t.replace(/\[programs\.localnet\][^\n]*/, (m) => m + `\nmerkle_distributor = "${programId}"`);
    }
  } else {
    t += `\n[programs.localnet]\nmerkle_distributor = "${programId}"\n`;
  }
  fs.writeFileSync(file, t);
}

function maybeReplaceIdlAddress(file: string, programId: string) {
  if (!fs.existsSync(file)) return;
  const idl = JSON.parse(fs.readFileSync(file, "utf-8"));
  idl.address = programId;
  fs.writeFileSync(file, JSON.stringify(idl, null, 2) + "\n");
}

(function main() {
  if (!fs.existsSync(KEYPAIR_PATH)) {
    console.error(`Missing ${KEYPAIR_PATH}. Run: anchor keys list or anchor build first.`);
    process.exit(1);
  }
  const pid = readProgramIdFromKeypair(KEYPAIR_PATH);
  replaceDeclareId(LIB_RS, pid);
  replaceAnchorTomlProgramId(ANCHOR_TOML, pid);
  maybeReplaceIdlAddress(IDL_JSON, pid);
  console.log(`Synced merkle_distributor program id -> ${pid}`);
})();

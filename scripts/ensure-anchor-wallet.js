const fs = require('fs');
const path = require('path');

const anchorToml = path.join(__dirname, '..', 'Anchor.toml');
let contents = fs.readFileSync(anchorToml, 'utf8');

// replace /Users/<user> with ~
contents = contents.replace(/\/Users\/[^"\s]+/g, '~');

const wallet = process.env.ANCHOR_WALLET || '~/.config/solana/id.json';

contents = contents.replace(/wallet\s*=\s*"[^"]*"/, `wallet  = "${wallet}"`);
if (contents.match(/upgrade_authority\s*=\s*"[^"]*"/)) {
  contents = contents.replace(/upgrade_authority\s*=\s*"[^"]*"/, `upgrade_authority = "${wallet}"`);
}

fs.writeFileSync(anchorToml, contents);

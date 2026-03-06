/**
 * Sync package.json version into tauri.conf.json and Cargo.toml. Run before tauri dev/build
 * so the app and Rust crate use the same version as package.json (single source of truth).
 */
import { readFileSync, writeFileSync } from "fs";
import { fileURLToPath } from "url";
import { dirname, join } from "path";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..");
const packagePath = join(root, "package.json");
const tauriConfPath = join(root, "src-tauri", "tauri.conf.json");
const cargoPath = join(root, "src-tauri", "Cargo.toml");

const pkg = JSON.parse(readFileSync(packagePath, "utf8"));
const version = pkg.version;
if (!version || typeof version !== "string") {
  throw new Error("package.json has no valid version");
}

// Tauri config
const tauriConf = JSON.parse(readFileSync(tauriConfPath, "utf8"));
tauriConf.version = version;
writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + "\n");

// Cargo.toml: replace the first version = "..." (in [package] section only)
const cargoContent = readFileSync(cargoPath, "utf8");
if (!/(^|\n)\s*version\s*=\s*"[^"]*"/.test(cargoContent)) {
  throw new Error("Could not find version line in src-tauri/Cargo.toml");
}
const newCargoContent = cargoContent.replace(
  /(^|\n)(\s*version\s*=\s*)"[^"]*"/,
  '$1$2"' + version + '"'
);
writeFileSync(cargoPath, newCargoContent);

console.log("Synced version:", version, "→ tauri.conf.json, Cargo.toml");

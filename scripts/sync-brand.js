/**
 * Sync brand.json into tauri.conf.json and capabilities. Run before tauri dev/build
 * so window title, product name, docs URL, etc. stay in sync with brand.json.
 */
import { readFileSync, writeFileSync } from "fs";
import { fileURLToPath } from "url";
import { dirname, join } from "path";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..");
const brandPath = join(root, "brand.json");
const tauriConfPath = join(root, "src-tauri", "tauri.conf.json");
const capabilitiesPath = join(root, "src-tauri", "capabilities", "default.json");

const brand = JSON.parse(readFileSync(brandPath, "utf8"));
const slug = (s) => s.toLowerCase().replace(/\s+/g, "-").replace(/[^a-z0-9-]/g, "");
const appSlug = slug(brand.appName);
const publisherSlug = slug(brand.publisher);
const identifier = `com.${publisherSlug}.${appSlug}`;

// Tauri config
const tauriConf = JSON.parse(readFileSync(tauriConfPath, "utf8"));
tauriConf.productName = brand.appName;
tauriConf.identifier = identifier;
if (tauriConf.app?.windows?.[0]) tauriConf.app.windows[0].title = brand.appName;
if (tauriConf.bundle) {
  if (brand.shortDescription) tauriConf.bundle.shortDescription = brand.shortDescription;
  if (brand.longDescription) tauriConf.bundle.longDescription = brand.longDescription;
}
if (tauriConf.plugins?.cli) {
  tauriConf.plugins.cli.description = `${brand.appName} – AWS Polly text-to-speech`;
}
writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + "\n");

// Capabilities: set opener allow URLs for docs from brand.docsUrl
const base = brand.docsUrl.replace(/\/$/, "");
const capabilities = JSON.parse(readFileSync(capabilitiesPath, "utf8"));
const perm = capabilities.permissions?.find((p) => p.identifier === "opener:allow-open-url");
if (perm?.allow) {
  const rest = perm.allow.filter((a) => !a.url?.startsWith(base));
  perm.allow = [{ url: base }, { url: `${base}/*` }, ...rest];
}
writeFileSync(capabilitiesPath, JSON.stringify(capabilities, null, 2) + "\n");

console.log("Synced brand:", brand.appName, "→ tauri.conf.json, capabilities");

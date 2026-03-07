/**
 * Sync brand.json into tauri.conf.json, capabilities, and flatpak/. Run before tauri dev/build
 * so window title, product name, docs URL, etc. stay in sync with brand.json.
 */
import { readFileSync, writeFileSync, mkdirSync } from "fs";
import { fileURLToPath } from "url";
import { dirname, join } from "path";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..");
const brandPath = join(root, "brand.json");
const tauriConfPath = join(root, "src-tauri", "tauri.conf.json");
const capabilitiesPath = join(root, "src-tauri", "capabilities", "default.json");
const flatpakDir = join(root, "flatpak");

const brand = JSON.parse(readFileSync(brandPath, "utf8"));
const slug = (s) => s.toLowerCase().replace(/\s+/g, "-").replace(/[^a-z0-9-]/g, "");
const pascalCase = (s) =>
  s
    .split(/\s+/)
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1).toLowerCase())
    .join("");
const appSlug = slug(brand.appName);
const publisherSlug = slug(brand.publisher);
const identifier = `com.${publisherSlug}.${appSlug}`;
const appId = `com.${publisherSlug}.${pascalCase(brand.appName)}`;

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

// Flatpak: generate manifest.yml and metainfo.xml from templates
const vars = {
  APP_ID: appId,
  APP_NAME: brand.appName,
  APP_SLUG: appSlug,
  SHORT_DESCRIPTION: brand.shortDescription || "",
  LONG_DESCRIPTION: brand.longDescription || "",
  DOCS_URL: brand.docsUrl || "",
  GITHUB_REPO: brand.githubRepo || "",
  DEVELOPER_NAME: brand.developerName || "Nicholas Santiago",
  PROJECT_LICENSE: brand.projectLicense || "GPL-3.0-or-later",
};
const replaceVars = (s) =>
  Object.entries(vars).reduce((acc, [k, v]) => acc.replace(new RegExp(`{{${k}}}`, "g"), v), s);

mkdirSync(flatpakDir, { recursive: true });
const manifestTemplate = readFileSync(join(flatpakDir, "manifest.yml.in"), "utf8");
const metainfoTemplate = readFileSync(join(flatpakDir, "metainfo.xml.in"), "utf8");
writeFileSync(join(flatpakDir, "manifest.yml"), replaceVars(manifestTemplate));
writeFileSync(join(flatpakDir, "metainfo.xml"), replaceVars(metainfoTemplate));
writeFileSync(join(flatpakDir, ".app-id"), appId);

console.log("Synced brand:", brand.appName, "→ tauri.conf.json, capabilities, flatpak/");

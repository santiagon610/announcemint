/**
 * Copy the built .deb to flatpak/{appSlug}.deb for flatpak-builder.
 * Run after `npm run tauri build -- --bundles deb`.
 */
import { readFileSync, copyFileSync, readdirSync } from "fs";
import { fileURLToPath } from "url";
import { dirname, join } from "path";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..");
const brand = JSON.parse(readFileSync(join(root, "brand.json"), "utf8"));
const slug = (s) => s.toLowerCase().replace(/\s+/g, "-").replace(/[^a-z0-9-]/g, "");
const appSlug = slug(brand.appName);

const debDir = join(root, "src-tauri", "target", "release", "bundle", "deb");
const debFiles = readdirSync(debDir).filter((f) => f.endsWith(".deb"));
if (debFiles.length === 0) {
  console.error("No .deb found. Run: npm run tauri build -- --bundles deb");
  process.exit(1);
}
const srcDeb = join(debDir, debFiles[0]);
const destDeb = join(root, "flatpak", `${appSlug}.deb`);
copyFileSync(srcDeb, destDeb);
console.log("Copied", debFiles[0], "→ flatpak/" + appSlug + ".deb");

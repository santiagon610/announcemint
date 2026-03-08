/**
 * Brand and app-wide configuration. Edit brand.json at project root to rebrand.
 */
import brand from "../brand.json";

export const PUBLISHER = brand.publisher as string;
export const APP_NAME = brand.appName as string;
export const HELP_DOCS_URL = brand.docsUrl as string;
export const GITHUB_REPO = brand.githubRepo as string;

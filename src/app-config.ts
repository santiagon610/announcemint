/**
 * Brand and app-wide configuration. Edit brand.json at project root to rebrand.
 */
import brand from "../brand.json";

export const PUBLISHER = brand.publisher as string;
export const APP_NAME = brand.appName as string;
export const HELP_DOCS_URL = brand.docsUrl as string;
export const GITHUB_REPO = brand.githubRepo as string;

/** When true, show the Help button in the header. Default false to hide for now. */
export const SHOW_HELP = false;
/** When true, show the About button in the footer and the About modal. Default false to hide for now. */
export const SHOW_ABOUT = false;

<script setup lang="ts">
import { ref, onMounted, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open, confirm } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import { relaunch } from "@tauri-apps/plugin-process";
import { APP_NAME, GITHUB_REPO, HELP_DOCS_URL } from "./app-config";

const outputDir = ref<string | null>(null);
const presetName = ref("OGG Vorbis");
const presets = ref<Array<{ name: string; format: string }>>([]);
const promptLines = ref("");
const voiceId = ref("Joanna");
const engine = ref<string>("neural");
const languageCode = ref("system");
const voices = ref<Array<{ id: string; name: string; language_code: string }>>(
  [],
);
const currentView = ref<"main" | "settings">("main");
const generating = ref(false);
const progress = ref<{ current: number; total: number } | null>(null);
const progressPromptName = ref("");
const progressStepMessage = ref("");
const error = ref("");
const sessionOk = ref<boolean | null>(null);

// AWS credentials
const defaultAwsConfigDir = ref<string | null>(null);
const awsConfigDir = ref<string | null>(null);
const awsProfile = ref("");
const awsProfilesList = ref<string[]>([]);
const awsUseManualCredentials = ref(false);
const awsRegionManual = ref("us-east-1");
const awsAccessKeyId = ref("");
const awsSecretAccessKey = ref("");
const awsProxyEnabled = ref(false);
const awsProxyProtocol = ref<"http" | "https" | "socks">("http");
const awsProxyHost = ref("");
const awsProxyPort = ref("");
const awsProxyUsername = ref("");
const awsProxyPassword = ref("");
const testingProxy = ref(false);
const proxyTestMessage = ref<{ ok: boolean; text: string } | null>(null);
interface PermissionStatus {
  name: string;
  granted: boolean;
  hint?: string;
}
interface CredentialsAndPermissionsResult {
  authenticated: boolean;
  error?: string;
  config_source?: string;
  region?: string;
  user_id?: string;
  account?: string;
  arn?: string;
  public_ip?: string;
  permissions: PermissionStatus[];
}
const credentialsAndPermissionsResult =
  ref<CredentialsAndPermissionsResult | null>(null);
const checkingCredentials = ref(false);
const awsRegionsList = ref<string[]>([]);
const showAbout = ref(false);
const appVersion = ref("");
const openDrawer = ref<"aws" | "voice" | "destination" | "dangerzone" | null>(
  null,
);
const openAwsSubdrawer = ref<"account" | "proxy" | null>(null);
const rememberPrompts = ref(true);
const promptFileNameFormat = ref("hyphen");

const promptFileNameFormatOptions = [
  {
    value: "none",
    label: 'Prompt name, no formatting changes (e.g. "North Dispatch 1.ogg")',
  },
  {
    value: "hyphen",
    label: 'Hyphens, unmodified case (e.g. "North-Dispatch-1.ogg")',
  },
  {
    value: "hyphen_lower",
    label: 'Hyphens, lower case (e.g. "north-dispatch-1.ogg")',
  },
  {
    value: "hyphen_upper",
    label: 'Hyphens, upper case (e.g. "NORTH-DISPATCH-1.ogg")',
  },
  {
    value: "underscore",
    label: 'Underscores, unmodified case (e.g. "North_Dispatch_1.ogg")',
  },
  {
    value: "underscore_lower",
    label: 'Underscores, lower case (e.g. "north_dispatch_1.ogg")',
  },
  {
    value: "underscore_upper",
    label: 'Underscores, upper case (e.g. "NORTH_DISPATCH_1.ogg")',
  },
];

const linesList = computed(() =>
  promptLines.value
    .split("\n")
    .map((l) => l.trim())
    .filter(Boolean),
);
const totalLines = computed(() => linesList.value.length);

function getSystemLanguageCode(): string {
  const locale =
    typeof navigator !== "undefined" ? navigator.language : "en-US";
  const lower = locale.toLowerCase();
  if (lower.startsWith("en-gb")) return "en-GB";
  if (lower.startsWith("en")) return "en-US";
  if (lower.startsWith("es")) return "es-US";
  if (lower.startsWith("fr")) return "fr-FR";
  if (lower.startsWith("de")) return "de-DE";
  return "en-US";
}
const effectiveLanguageCode = computed(() =>
  languageCode.value === "system"
    ? getSystemLanguageCode()
    : languageCode.value,
);

const authStatusIndicator = computed<"green" | "yellow" | "red" | null>(() => {
  const r = credentialsAndPermissionsResult.value;
  if (!r) return null;
  if (!r.authenticated) return "red";
  const allGranted =
    r.permissions.length > 0 && r.permissions.every((p) => p.granted);
  return allGranted ? "green" : "yellow";
});

const authStatusTooltip = computed(() => {
  const s = authStatusIndicator.value;
  if (s === "green")
    return "Able to authenticate to AWS and all needed IAM permissions are granted";
  if (s === "yellow")
    return "Able to authenticate to AWS, but not all necessary permissions are granted";
  if (s === "red") return "Unable to connect or authenticate to AWS";
  return "AWS status unknown";
});

async function loadPresets() {
  try {
    presets.value = await invoke("list_presets");
  } catch (e) {
    console.error(e);
  }
}

async function checkSession() {
  try {
    await invoke("polly_check_session");
    sessionOk.value = true;
    error.value = "";
  } catch (e) {
    sessionOk.value = false;
    error.value = "No valid AWS session. Add credentials in Settings.";
  }
}

async function loadVoices() {
  try {
    voices.value = await invoke("polly_describe_voices", {
      languageCode: effectiveLanguageCode.value || null,
      engine: engine.value || null,
    });
    if (
      voiceId.value &&
      voices.value.length &&
      !voices.value.some((v) => v.id === voiceId.value)
    ) {
      voiceId.value = voices.value[0].id;
    }
  } catch (e) {
    console.error(e);
  }
}

async function pickDir() {
  const selected = await open({
    directory: true,
    multiple: false,
  });
  if (selected && typeof selected === "string") {
    outputDir.value = selected;
  }
}

async function pickAwsConfigDir() {
  const selected = await open({
    directory: true,
    multiple: false,
  });
  if (selected && typeof selected === "string") {
    awsConfigDir.value = selected;
    await loadAwsProfiles();
  }
}

async function loadAwsProfiles() {
  try {
    awsProfilesList.value = await invoke<string[]>("list_aws_profiles", {
      configDir: awsConfigDir.value,
    });
    if (
      awsProfilesList.value.length &&
      !awsProfilesList.value.includes(awsProfile.value)
    ) {
      awsProfile.value = awsProfilesList.value[0];
    }
  } catch (e) {
    console.error(e);
    awsProfilesList.value = [];
  }
}

async function checkCredentialsAndPermissions() {
  credentialsAndPermissionsResult.value = null;
  checkingCredentials.value = true;
  try {
    const result = await invoke<CredentialsAndPermissionsResult>(
      "check_credentials_and_permissions",
    );
    credentialsAndPermissionsResult.value = result;
  } catch (e) {
    credentialsAndPermissionsResult.value = {
      authenticated: false,
      error: String(e),
      permissions: [
        { name: "polly:DescribeVoices", granted: false },
        { name: "polly:SynthesizeSpeech", granted: false },
      ],
    };
  } finally {
    checkingCredentials.value = false;
  }
}

async function testProxyConfig() {
  proxyTestMessage.value = null;
  testingProxy.value = true;
  try {
    await invoke("test_proxy_config");
    proxyTestMessage.value = {
      ok: true,
      text: "Proxy configuration is working. Successfully reached the AWS API in the selected region.",
    };
  } catch (e) {
    proxyTestMessage.value = {
      ok: false,
      text: String(e),
    };
  } finally {
    testingProxy.value = false;
  }
}

function openExternal(e: MouseEvent) {
  const target = e.currentTarget as HTMLAnchorElement;
  const url = target?.href;
  if (url) {
    e.preventDefault();
    openUrl(url).catch(console.error);
  }
}

function stepToLabel(step: string): string {
  switch (step) {
    case "submitted":
      return "📤 Submitted prompt to Amazon Polly";
    case "downloading":
      return "⬇️ Downloading prompt audio";
    case "converting":
      return "🔄 Converting to destination format";
    case "no_conversion":
      return "🔄 No conversion (OGG)";
    case "saving":
      return "💾 Saving";
    default:
      return step;
  }
}

async function generate() {
  error.value = "";
  if (!outputDir.value) {
    error.value = "Select an output directory first.";
    return;
  }
  if (totalLines.value === 0) {
    error.value = "Enter at least one prompt (one per line).";
    return;
  }
  generating.value = true;
  progress.value = { current: 0, total: totalLines.value };
  progressPromptName.value = "";
  progressStepMessage.value = "";
  let unlisten: (() => void) | undefined;
  try {
    unlisten = await listen<{
      prompt_name: string;
      current: number;
      total: number;
      step: string;
    }>("generate-progress", (event) => {
      const { prompt_name, current, total, step } = event.payload;
      progress.value = { current, total };
      progressPromptName.value = prompt_name;
      progressStepMessage.value = stepToLabel(step);
    });
    await checkSession();
    if (!sessionOk.value) return;
    const existing = await invoke<string[]>("check_destination_paths", {
      lines: linesList.value,
      outputDir: outputDir.value,
      presetName: presetName.value || null,
    });
    if (existing.length > 0) {
      const overwrite = await confirm(
        `${existing.length} file(s) already exist and will be overwritten. Continue?`,
        {
          title: "Overwrite files?",
          kind: "warning",
          okLabel: "Overwrite",
          cancelLabel: "Cancel",
        },
      );
      if (!overwrite) {
        return;
      }
    }
    const paths = await invoke<string[]>("polly_generate_prompts", {
      lines: linesList.value,
      voiceId: voiceId.value,
      engine: engine.value || null,
      outputDir: outputDir.value,
      presetName: presetName.value || null,
    });
    progress.value = { current: paths.length, total: totalLines.value };
    error.value = "";
  } catch (e) {
    error.value = String(e);
  } finally {
    unlisten?.();
    generating.value = false;
    progress.value = null;
    progressPromptName.value = "";
    progressStepMessage.value = "";
  }
}

async function loadConfig() {
  try {
    const c = await invoke<{
      voice_id?: string;
      engine?: string;
      language_code?: string;
      preset_name?: string;
      output_dir?: string;
      prompt_lines?: string;
      remember_prompts?: boolean;
      prompt_file_name_format?: string;
      aws_proxy_enabled?: boolean;
      aws_proxy_url?: string;
      aws_proxy_protocol?: string;
      aws_proxy_host?: string;
      aws_proxy_port?: string;
      aws_proxy_username?: string;
      aws_proxy_password?: string;
      aws_profile?: string;
      aws_config_dir?: string;
      aws_region_manual?: string;
      aws_access_key_id?: string;
      aws_secret_access_key?: string;
      aws_use_manual?: boolean;
    }>("get_config");
    if (c?.language_code) languageCode.value = c.language_code;
    else languageCode.value = "system";
    const effective =
      languageCode.value === "system"
        ? getSystemLanguageCode()
        : languageCode.value;
    if (c?.voice_id) voiceId.value = c.voice_id;
    else voiceId.value = effective === "en-US" ? "Joanna" : "";
    if (c?.engine) engine.value = c.engine;
    else engine.value = "neural";
    if (c?.preset_name) presetName.value = c.preset_name;
    else presetName.value = "OGG Vorbis";
    if (c?.output_dir) outputDir.value = c.output_dir;
    if (c?.remember_prompts != null) rememberPrompts.value = c.remember_prompts;
    else rememberPrompts.value = true;
    if (rememberPrompts.value && c?.prompt_lines != null)
      promptLines.value = c.prompt_lines;
    if (c?.prompt_file_name_format != null)
      promptFileNameFormat.value = c.prompt_file_name_format;
    else promptFileNameFormat.value = "hyphen";
    if (c?.aws_proxy_enabled != null)
      awsProxyEnabled.value = c.aws_proxy_enabled;
    else awsProxyEnabled.value = false;
    if (c?.aws_proxy_protocol === "https" || c?.aws_proxy_protocol === "socks")
      awsProxyProtocol.value = c.aws_proxy_protocol;
    else awsProxyProtocol.value = "http";
    if (c?.aws_proxy_host != null) awsProxyHost.value = c.aws_proxy_host;
    else awsProxyHost.value = "";
    if (c?.aws_proxy_port != null) awsProxyPort.value = c.aws_proxy_port;
    else awsProxyPort.value = "";
    if (c?.aws_proxy_username != null)
      awsProxyUsername.value = c.aws_proxy_username;
    else awsProxyUsername.value = "";
    if (c?.aws_proxy_password != null)
      awsProxyPassword.value = c.aws_proxy_password;
    else awsProxyPassword.value = "";
    if (c?.aws_profile != null) awsProfile.value = c.aws_profile;
    if (c?.aws_config_dir != null) awsConfigDir.value = c.aws_config_dir;
    if (c?.aws_region_manual != null)
      awsRegionManual.value = c.aws_region_manual;
    else awsRegionManual.value = "us-east-1";
    if (c?.aws_access_key_id != null)
      awsAccessKeyId.value = c.aws_access_key_id;
    if (c?.aws_secret_access_key != null)
      awsSecretAccessKey.value = c.aws_secret_access_key;
    if (c?.aws_use_manual != null)
      awsUseManualCredentials.value = c.aws_use_manual;
    else
      awsUseManualCredentials.value = !!(
        c?.aws_access_key_id && c?.aws_secret_access_key
      );
    if (!defaultAwsConfigDir.value) {
      defaultAwsConfigDir.value =
        (await invoke<string | null>("get_default_aws_config_dir")) ?? null;
    }
    await loadAwsProfiles();
    if (!c?.aws_profile) {
      const envProfile = await invoke<string | null>("get_aws_profile_env");
      if (envProfile && awsProfilesList.value.includes(envProfile)) {
        awsProfile.value = envProfile;
      }
    }
  } catch (e) {
    console.error(e);
  }
}

function saveConfig() {
  invoke("save_config", {
    config: {
      voice_id: voiceId.value,
      engine: engine.value,
      language_code: languageCode.value,
      preset_name: presetName.value,
      output_dir: outputDir.value,
      prompt_lines: rememberPrompts.value ? promptLines.value || null : null,
      remember_prompts: rememberPrompts.value,
      prompt_file_name_format: promptFileNameFormat.value || null,
      aws_proxy_enabled: awsProxyEnabled.value,
      aws_proxy_protocol: awsProxyProtocol.value || null,
      aws_proxy_host: awsProxyHost.value || null,
      aws_proxy_port: awsProxyPort.value || null,
      aws_proxy_username: awsProxyUsername.value || null,
      aws_proxy_password: awsProxyPassword.value || null,
      aws_profile: awsProfile.value || null,
      aws_config_dir: awsConfigDir.value,
      aws_region_manual: awsRegionManual.value || null,
      aws_access_key_id: awsAccessKeyId.value || null,
      aws_secret_access_key: awsSecretAccessKey.value || null,
      aws_use_manual: awsUseManualCredentials.value,
    },
  }).catch(console.error);
}

watch(
  [
    voiceId,
    engine,
    languageCode,
    presetName,
    outputDir,
    promptLines,
    rememberPrompts,
    promptFileNameFormat,
    awsProxyEnabled,
    awsProxyProtocol,
    awsProxyHost,
    awsProxyPort,
    awsProxyUsername,
    awsProxyPassword,
    awsProfile,
    awsConfigDir,
    awsRegionManual,
    awsAccessKeyId,
    awsSecretAccessKey,
    awsUseManualCredentials,
  ],
  () => {
    saveConfig();
  },
  { deep: true },
);

watch(
  [
    awsProxyEnabled,
    awsProxyProtocol,
    awsProxyHost,
    awsProxyPort,
    awsProxyUsername,
    awsProxyPassword,
  ],
  () => {
    proxyTestMessage.value = null;
  },
);

/** Sync window theme after mount so app content follows dark/light. Runs in Vue lifecycle so it cannot block initial render. On Linux uses get_system_theme (XDG portal). */
function minimizeWindow(): void {
  getCurrentWindow().minimize();
}
function toggleMaximizeWindow(): void {
  getCurrentWindow().toggleMaximize();
}
function closeWindow(): void {
  getCurrentWindow().close();
}

const resettingConfig = ref(false);

async function resetConfigAndRelaunch(): Promise<void> {
  const confirmed = await confirm(
    "This will permanently delete your config file and restart the app. All saved settings (voice, AWS profile, output folder, etc.) will be lost. Only do this if you are sure.",
    {
      title: "Reset settings?",
      kind: "warning",
      okLabel: "Delete config and restart",
      cancelLabel: "Cancel",
    },
  );
  if (!confirmed) return;
  try {
    resettingConfig.value = true;
    await invoke("delete_config_file");
    await relaunch();
  } catch (e) {
    error.value = String(e);
    resettingConfig.value = false;
  }
}

function initThemeSync(): void {
  (async () => {
    const log = (msg: string, ...args: unknown[]) =>
      console.log("[theme]", msg, ...args);
    try {
      const win = getCurrentWindow();
      const setTheme = (theme: string | null, source?: string) => {
        const v = theme === "dark" ? "dark" : "light";
        document.documentElement.dataset.theme = v;
        if (source) log("set theme:", v, `(${source})`);
      };
      const portalTheme = await invoke<string | null>("get_system_theme").catch(
        (e) => {
          log("get_system_theme failed, using window theme", e);
          return null;
        },
      );
      if (portalTheme === "dark" || portalTheme === "light") {
        setTheme(portalTheme, "portal");
      } else {
        const windowTheme = await win.theme().catch((e) => {
          log("window.theme() failed", e);
          return null;
        });
        setTheme(windowTheme, "window");
      }
      await win.onThemeChanged((ev) =>
        setTheme(ev.payload, "onThemeChanged"),
      );
      log("listening for theme changes");
    } catch (e) {
      console.warn("[theme] sync failed, using prefers-color-scheme", e);
    }
  })();
}

onMounted(async () => {
  document.title = APP_NAME;
  initThemeSync();
  try {
    awsRegionsList.value = await invoke<string[]>("list_aws_regions");
  } catch (e) {
    console.error(e);
  }
  try {
    appVersion.value = await invoke<string>("get_app_version");
  } catch (e) {
    console.error(e);
  }
  loadPresets();
  loadConfig().then(() => {
    loadVoices();
    checkSession();
    checkCredentialsAndPermissions();
  });
});
</script>

<template>
  <div class="app">
    <header class="titlebar">
      <div class="titlebar-drag" data-tauri-drag-region>{{ APP_NAME }}</div>
      <div class="titlebar-controls">
        <button
          type="button"
          class="titlebar-btn"
          title="Minimize"
          @click="minimizeWindow"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M19 13H5v-2h14z"/></svg>
        </button>
        <button
          type="button"
          class="titlebar-btn"
          title="Maximize"
          @click="toggleMaximizeWindow"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M4 4h16v16H4zm2 2v12h12V6z"/></svg>
        </button>
        <button
          type="button"
          class="titlebar-btn titlebar-btn-close"
          title="Close"
          @click="closeWindow"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/></svg>
        </button>
      </div>
    </header>
    <!-- Main view: prompts + generate -->
    <template v-if="currentView === 'main'">
      <header class="header">
        <img src="/icon.svg" alt="" class="header-icon" />
        <h1>{{ APP_NAME }}</h1>
        <span class="header-status-wrap">
          <span
            v-if="authStatusIndicator"
            class="auth-status-dot"
            :class="authStatusIndicator"
            :title="authStatusTooltip"
          >
            {{
              authStatusIndicator === "green"
                ? "🟢"
                : authStatusIndicator === "yellow"
                  ? "🟡"
                  : "🔴"
            }}
          </span>
          <button
            type="button"
            class="btn-secondary"
            @click="openUrl(HELP_DOCS_URL).catch(console.error)"
          >
            Help
          </button>
          <button
            type="button"
            class="btn-secondary"
            @click="currentView = 'settings'"
          >
            Settings
          </button>
        </span>
      </header>

      <main class="main">
        <div class="form-row">
          <label>Prompts (one per line)</label>
          <textarea
            v-model="promptLines"
            class="prompts-input"
            placeholder="Primary Zone&#10;Dispatch&#10;North Police 1&#10;North Police 2&#10;South Police 1&#10;South Police 2&#10;..."
            rows="12"
          />
        </div>
        <div class="actions">
          <button
            type="button"
            class="btn-primary"
            :disabled="generating || !outputDir || totalLines === 0 || !voiceId"
            @click="generate"
          >
            {{ generating ? "Generating…" : "Generate Prompts" }}
          </button>
        </div>
        <div v-if="progress" class="progress-panel">
          <p v-if="progressPromptName" class="progress-prompt-name">
            {{ progressPromptName }}
          </p>
          <div class="progress-bar-wrap">
            <div
              class="progress-bar-fill"
              :style="{
                width: progress.total
                  ? `${(100 * progress.current) / progress.total}%`
                  : '0%',
              }"
            />
          </div>
          <p class="progress-count">
            {{ progress.current }} / {{ progress.total }}
          </p>
          <p v-if="progressStepMessage" class="progress-step-message">
            {{ progressStepMessage }}
          </p>
        </div>
        <p v-if="error" class="error">{{ error }}</p>
      </main>

      <footer class="footer">
        <button type="button" class="footer-link" @click="showAbout = true">
          About
        </button>
      </footer>
    </template>

    <!-- Settings: full page with two drawers -->
    <template v-if="currentView === 'settings'">
      <header class="header settings-header">
        <h1 class="settings-title">Settings</h1>
        <button type="button" class="btn-primary" @click="currentView = 'main'">
          Return
        </button>
      </header>

      <div class="settings-page">
        <!-- Drawer 1: Connecting to AWS -->
        <section class="drawer accordion-drawer">
          <button
            type="button"
            class="drawer-header"
            :aria-expanded="openDrawer === 'aws'"
            @click="openDrawer = openDrawer === 'aws' ? null : 'aws'"
          >
            <h2 class="drawer-title">Connecting to AWS</h2>
            <span
              class="drawer-chevron"
              :class="{ open: openDrawer === 'aws' }"
              aria-hidden="true"
              >▼</span
            >
          </button>
          <div v-show="openDrawer === 'aws'" class="drawer-body drawer-body-no-border">
            <!-- Subdrawer: AWS Account -->
            <div class="subdrawer">
              <button
                type="button"
                class="subdrawer-header"
                :aria-expanded="openAwsSubdrawer === 'account'"
                @click="
                  openAwsSubdrawer =
                    openAwsSubdrawer === 'account' ? null : 'account'
                "
              >
                <span class="subdrawer-title">AWS Account</span>
                <span
                  class="drawer-chevron"
                  :class="{ open: openAwsSubdrawer === 'account' }"
                  aria-hidden="true"
                  >▼</span
                >
              </button>
              <div
                v-show="openAwsSubdrawer === 'account'"
                class="subdrawer-body"
              >
                <div class="form-row credential-toggle-row">
                  <label class="toggle-label">Credential source</label>
                  <div class="toggle-buttons">
                <button
                  type="button"
                  :class="[
                    'btn-secondary',
                    'toggle-btn',
                    { active: !awsUseManualCredentials },
                  ]"
                  @click="awsUseManualCredentials = false"
                >
                  AWS config file
                </button>
                <button
                  type="button"
                  :class="[
                    'btn-secondary',
                    'toggle-btn',
                    { active: awsUseManualCredentials },
                  ]"
                  @click="awsUseManualCredentials = true"
                >
                  Manual credentials
                </button>
                  </div>
                </div>

                <template v-if="awsUseManualCredentials">
              <div class="drawer-fields">
                <div class="form-row">
                  <label>AWS Region</label>
                  <select v-model="awsRegionManual">
                    <option v-for="r in awsRegionsList" :key="r" :value="r">
                      {{ r }}
                    </option>
                  </select>
                </div>
                <div class="form-row">
                  <label>Access Key ID</label>
                  <input
                    v-model="awsAccessKeyId"
                    type="text"
                    placeholder="AKIA…"
                    autocomplete="off"
                  />
                </div>
                <div class="form-row">
                  <label>Secret Access Key</label>
                  <input
                    v-model="awsSecretAccessKey"
                    type="password"
                    placeholder="…"
                    autocomplete="off"
                  />
                </div>
              </div>
            </template>
            <template v-else>
              <div class="drawer-fields">
                <div class="form-row">
                  <label>Config directory</label>
                  <div class="row-actions row-actions-wrap">
                    <span class="config-dir-display">{{
                      awsConfigDir || defaultAwsConfigDir || "Default (~/.aws)"
                    }}</span>
                    <span class="row-buttons">
                      <button
                        type="button"
                        class="btn-secondary"
                        @click="pickAwsConfigDir"
                      >
                        Browse…
                      </button>
                      <button
                        v-if="awsConfigDir"
                        type="button"
                        class="btn-link btn-link-small"
                        @click="
                          awsConfigDir = null;
                          loadAwsProfiles();
                        "
                      >
                        Use default
                      </button>
                    </span>
                  </div>
                </div>
                <div class="form-row">
                  <label>Profile</label>
                  <select v-model="awsProfile">
                    <option value="">— Use default / environment —</option>
                    <option v-for="p in awsProfilesList" :key="p" :value="p">
                      {{ p }}
                    </option>
                  </select>
                </div>
              </div>
            </template>

            <div class="form-row credentials-check-row">
              <button
                type="button"
                class="btn-secondary"
                :disabled="checkingCredentials"
                @click="checkCredentialsAndPermissions"
              >
                {{
                  checkingCredentials
                    ? "Checking…"
                    : "Check credentials and permissions"
                }}
              </button>
            </div>
            <div
              v-if="credentialsAndPermissionsResult"
              class="credentials-result"
            >
              <p
                v-if="credentialsAndPermissionsResult.error"
                class="credentials-error"
              >
                {{ credentialsAndPermissionsResult.error }}
              </p>
              <table class="credentials-table">
                <tbody>
                  <tr>
                    <th>Config file</th>
                    <td class="font-mono">
                      {{ credentialsAndPermissionsResult.config_source ?? "—" }}
                    </td>
                  </tr>
                  <tr>
                    <th>Region</th>
                    <td>{{ credentialsAndPermissionsResult.region ?? "—" }}</td>
                  </tr>
                  <tr>
                    <th>User ID</th>
                    <td>
                      {{ credentialsAndPermissionsResult.user_id ?? "—" }}
                    </td>
                  </tr>
                  <tr>
                    <th>Account</th>
                    <td>
                      {{ credentialsAndPermissionsResult.account ?? "—" }}
                    </td>
                  </tr>
                  <tr>
                    <th>ARN</th>
                    <td class="arn-cell">
                      <span class="arn-text font-mono">{{
                        credentialsAndPermissionsResult.arn ?? "—"
                      }}</span>
                    </td>
                  </tr>
                  <tr>
                    <th>Public IP</th>
                    <td>
                      {{ credentialsAndPermissionsResult.public_ip ?? "—" }}
                    </td>
                  </tr>
                  <tr
                    v-for="p in credentialsAndPermissionsResult.permissions"
                    :key="p.name"
                  >
                    <th>{{ p.name }}</th>
                    <td>
                      <span>{{ p.granted ? "✅" : "❌" }}</span>
                      <span v-if="!p.granted && p.hint" class="permission-hint">
                        {{ p.hint }}</span
                      >
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
            <p v-if="sessionOk === false" class="session-hint">
              Configure AWS credentials above, or set AWS_PROFILE /
              ~/.aws/credentials.
            </p>
              </div>
            </div>

            <!-- Subdrawer: Network Proxy -->
            <div class="subdrawer">
              <button
                type="button"
                class="subdrawer-header"
                :aria-expanded="openAwsSubdrawer === 'proxy'"
                @click="
                  openAwsSubdrawer =
                    openAwsSubdrawer === 'proxy' ? null : 'proxy'
                "
              >
                <span class="subdrawer-title">Network Proxy</span>
                <span
                  class="drawer-chevron"
                  :class="{ open: openAwsSubdrawer === 'proxy' }"
                  aria-hidden="true"
                  >▼</span
                >
              </button>
              <div
                v-show="openAwsSubdrawer === 'proxy'"
                class="subdrawer-body"
              >
                <div class="form-row form-row-checkbox proxy-toggle-row">
                  <label class="checkbox-label">
                    <input v-model="awsProxyEnabled" type="checkbox" />
                    Use an HTTP(S) or SOCKS proxy to access AWS
                  </label>
                </div>
                <div v-show="awsProxyEnabled" class="drawer-fields proxy-fields">
                  <div class="form-row">
                    <label>Proxy Protocol</label>
                    <select v-model="awsProxyProtocol">
                      <option value="http">HTTP</option>
                      <option value="https">HTTPS</option>
                      <option value="socks">SOCKS</option>
                    </select>
                  </div>
                  <div class="form-row">
                    <label>Proxy Host</label>
                    <input
                      v-model="awsProxyHost"
                      type="text"
                      placeholder="FQDN or IP address"
                      autocomplete="off"
                    />
                  </div>
                  <div class="form-row">
                    <label>Proxy Port</label>
                    <input
                      v-model="awsProxyPort"
                      type="number"
                      placeholder="e.g. 8080"
                      min="1"
                      max="65535"
                      autocomplete="off"
                    />
                  </div>
                  <div class="form-row">
                    <label>Proxy Username</label>
                    <input
                      v-model="awsProxyUsername"
                      type="text"
                      placeholder="Optional"
                      autocomplete="off"
                    />
                  </div>
                  <div class="form-row">
                    <label>Proxy Password</label>
                    <input
                      v-model="awsProxyPassword"
                      type="password"
                      placeholder="Optional"
                      autocomplete="new-password"
                    />
                  </div>
                  <div class="form-row credentials-check-row">
                    <button
                      type="button"
                      class="btn-secondary"
                      :disabled="testingProxy"
                      @click="testProxyConfig"
                    >
                      {{ testingProxy ? "Testing…" : "Test proxy" }}
                    </button>
                  </div>
                  <p
                    v-if="proxyTestMessage"
                    :class="
                      proxyTestMessage.ok
                        ? 'proxy-test-ok'
                        : 'credentials-error'
                    "
                  >
                    {{ proxyTestMessage.text }}
                  </p>
                </div>
              </div>
            </div>
          </div>
        </section>

        <!-- Drawer 2: Voice Options -->
        <section class="drawer accordion-drawer">
          <button
            type="button"
            class="drawer-header"
            :aria-expanded="openDrawer === 'voice'"
            @click="openDrawer = openDrawer === 'voice' ? null : 'voice'"
          >
            <h2 class="drawer-title">Voice Options</h2>
            <span
              class="drawer-chevron"
              :class="{ open: openDrawer === 'voice' }"
              aria-hidden="true"
              >▼</span
            >
          </button>
          <div v-show="openDrawer === 'voice'" class="drawer-body">
            <div class="drawer-fields">
              <div class="form-row">
                <label>Language</label>
                <select v-model="languageCode" @change="loadVoices">
                  <option value="system">System locale</option>
                  <option value="en-US">English (US)</option>
                  <option value="en-GB">English (UK)</option>
                  <option value="es-US">Spanish (US)</option>
                  <option value="fr-FR">French (France)</option>
                  <option value="de-DE">German</option>
                </select>
              </div>
              <div class="form-row">
                <label>Engine</label>
                <select v-model="engine" @change="loadVoices">
                  <option value="standard">Standard</option>
                  <option value="neural">Neural</option>
                </select>
              </div>
              <div class="form-row">
                <label>Voice</label>
                <select v-model="voiceId">
                  <option value="">— Select voice —</option>
                  <option v-for="v in voices" :key="v.id" :value="v.id">
                    {{ v.name }} ({{ v.language_code }})
                  </option>
                </select>
              </div>
              <div class="form-row">
                <label>Output preset</label>
                <select v-model="presetName">
                  <option v-for="p in presets" :key="p.name" :value="p.name">
                    {{ p.name }}
                  </option>
                </select>
              </div>
              <div class="form-row form-row-checkbox">
                <label class="checkbox-label">
                  <input v-model="rememberPrompts" type="checkbox" />
                  Remember prompt names after closing
                </label>
              </div>
            </div>
          </div>
        </section>

        <!-- Drawer 3: Saving Prompts -->
        <section class="drawer accordion-drawer">
          <button
            type="button"
            class="drawer-header"
            :aria-expanded="openDrawer === 'destination'"
            @click="
              openDrawer = openDrawer === 'destination' ? null : 'destination'
            "
          >
            <h2 class="drawer-title">Saving Prompts</h2>
            <span
              class="drawer-chevron"
              :class="{ open: openDrawer === 'destination' }"
              aria-hidden="true"
              >▼</span
            >
          </button>
          <div v-show="openDrawer === 'destination'" class="drawer-body">
            <div class="drawer-fields drawer-fields-full">
              <div class="form-row">
                <label>Output directory</label>
                <button
                  type="button"
                  class="btn-secondary btn-full"
                  @click="pickDir"
                >
                  {{ outputDir || "Choose folder…" }}
                </button>
              </div>
              <div class="form-row">
                <label>Filename Formatting</label>
                <select v-model="promptFileNameFormat">
                  <option
                    v-for="opt in promptFileNameFormatOptions"
                    :key="opt.value"
                    :value="opt.value"
                  >
                    {{ opt.label }}
                  </option>
                </select>
              </div>
            </div>
          </div>
        </section>

        <!-- Danger Zone -->
        <section class="drawer accordion-drawer drawer-danger">
          <button
            type="button"
            class="drawer-header"
            :aria-expanded="openDrawer === 'dangerzone'"
            @click="
              openDrawer = openDrawer === 'dangerzone' ? null : 'dangerzone'
            "
          >
            <h2 class="drawer-title">Danger Zone</h2>
            <span
              class="drawer-chevron"
              :class="{ open: openDrawer === 'dangerzone' }"
              aria-hidden="true"
              >▼</span
            >
          </button>
          <div v-show="openDrawer === 'dangerzone'" class="drawer-body">
            <div class="drawer-fields">
              <p class="danger-zone-desc">
                Permanently delete the app config file and restart. All saved
                settings will be lost.
              </p>
              <div class="form-row">
                <button
                  type="button"
                  class="btn-danger"
                  :disabled="resettingConfig"
                  @click="resetConfigAndRelaunch"
                >
                  {{ resettingConfig ? "Resetting…" : "Delete config and restart" }}
                </button>
              </div>
            </div>
          </div>
        </section>
      </div>
    </template>

    <!-- About modal (global) -->
    <div
      v-show="showAbout"
      class="about-overlay"
      role="dialog"
      aria-modal="true"
      aria-labelledby="about-title"
      @click.self="showAbout = false"
    >
      <div class="about-panel" @click.stop>
        <h2 id="about-title" class="about-title">{{ APP_NAME }}</h2>
        <p class="about-version">Version {{ appVersion || "…" }}</p>
        <p class="about-copyright">Nicholas Santiago</p>
        <p class="about-license">
          This program is free software: you can redistribute it and/or modify
          it under the terms of the
          <a
            href="https://www.gnu.org/licenses/gpl-3.0.html"
            @click="openExternal"
            >GNU General Public License</a
          >
          as published by the Free Software Foundation, version 3 or later. See
          the LICENSE file in the source distribution.
        </p>
        <p class="about-warranty">
          This program is distributed in the hope that it will be useful, but
          WITHOUT ANY WARRANTY.
        </p>
        <p class="about-github">
          <a :href="GITHUB_REPO" @click="openExternal">GitHub</a>
        </p>
        <button
          type="button"
          class="btn-secondary about-close"
          @click="showAbout = false"
        >
          Close
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.app {
  max-width: 56rem;
  margin: 0 auto;
  padding: 1.5rem;
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}
.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
  gap: 0.75rem;
}
.header-icon {
  width: 2rem;
  height: 2rem;
  flex-shrink: 0;
}
.header h1 {
  font-size: 1.5rem;
  margin: 0;
  flex: 1;
}
.header-status-wrap {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
.auth-status-dot {
  font-size: 0.75rem;
  line-height: 1;
}
.settings-header {
  margin-bottom: 1.5rem;
}
.settings-title {
  font-size: 1.5rem;
  margin: 0;
  flex: 1;
}
.settings-page {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}
.drawer {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 0.5rem;
  padding: 1.25rem;
}
.accordion-drawer {
  padding: 0;
  overflow: hidden;
}
.drawer-header {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  padding: 1rem 1.25rem;
  background: none;
  border: none;
  cursor: pointer;
  font: inherit;
  color: inherit;
  text-align: left;
}
.drawer-header:hover {
  background: var(--color-bg);
}
.drawer-header .drawer-title {
  margin: 0;
  flex: 1;
}
.drawer-chevron {
  font-size: 0.75rem;
  opacity: 0.7;
  transform: rotate(-90deg);
  transition: transform 0.2s ease;
}
.drawer-chevron.open {
  transform: rotate(0deg);
}
.drawer-body {
  padding: 0 1.25rem 1.25rem;
  border-top: 1px solid var(--color-border);
}
.drawer-body-no-border {
  border-top: none;
}
.subdrawer {
  border: 1px solid var(--color-border);
  border-radius: 0.375rem;
  overflow: hidden;
  margin-bottom: 0.75rem;
}
.subdrawer:last-child {
  margin-bottom: 0;
}
.subdrawer-header {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
  background: none;
  border: none;
  cursor: pointer;
  font: inherit;
  color: inherit;
  text-align: left;
}
.subdrawer-header:hover {
  background: var(--color-bg);
}
.subdrawer-title {
  font-size: 0.9375rem;
  font-weight: 600;
  color: var(--color-text);
}
.subdrawer-body {
  padding: 0 1rem 1rem;
  border-top: 1px solid var(--color-border);
}
.drawer-title {
  font-size: 1rem;
  font-weight: 600;
  margin: 0 0 1rem 0;
  color: var(--color-text);
}
.drawer-section-title {
  font-size: 0.9375rem;
  font-weight: 600;
  margin: 0 0 0.75rem 0;
  color: var(--color-text);
}
.drawer-separator {
  border: none;
  border-top: 1px solid var(--color-border);
  margin: 1.25rem 0;
}
.permission-hint {
  font-size: 0.8125rem;
  color: var(--color-text-muted, #666);
  margin-left: 0.25rem;
}
.drawer-fields {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  margin-bottom: 1rem;
}
.credential-toggle-row {
  margin-bottom: 1rem;
}
.toggle-label {
  display: block;
  margin-bottom: 0.375rem;
}
.toggle-buttons {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
}
.toggle-btn {
  flex: 1;
  min-width: 8rem;
}
.toggle-btn.active {
  background: var(--color-primary);
  color: white;
  border-color: var(--color-primary);
}
.toggle-btn.active:hover {
  background: var(--color-primary-hover);
  border-color: var(--color-primary-hover);
}
.drawer-fields input:not([type="checkbox"]):not([type="radio"]),
.drawer-fields select {
  padding: 0.375rem 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: 0.375rem;
  font-size: 0.875rem;
  width: 100%;
  max-width: 20rem;
}
.drawer-fields-full select,
.drawer-fields-full input:not([type="checkbox"]):not([type="radio"]) {
  max-width: none;
}
.btn-full {
  width: 100%;
  text-align: left;
}
.main {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}
.form-row {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}
.form-row label {
  font-size: 0.875rem;
  font-weight: 500;
}
.row-actions {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}
.row-actions-wrap {
  flex-wrap: wrap;
  gap: 0.5rem;
}
.row-buttons {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}
.btn-link-small {
  font-size: 0.8125rem;
}
.prompts-input {
  width: 100%;
  min-height: 12rem;
  resize: vertical;
  font-family: inherit;
}
.actions {
  margin-top: 0.5rem;
}
.btn-primary {
  padding: 0.75rem 1.5rem;
  font-size: 1rem;
  font-weight: 600;
  border-radius: 0.5rem;
}
.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
.btn-secondary {
  background: var(--color-surface);
  color: var(--color-text);
  border: 1px solid var(--color-border);
  padding: 0.5rem 0.75rem;
  border-radius: 0.375rem;
}
.btn-secondary:hover {
  background: var(--color-border);
}
.btn-danger {
  background: var(--color-error);
  color: #fff;
  border: 1px solid var(--color-error);
  padding: 0.5rem 0.75rem;
  border-radius: 0.375rem;
  cursor: pointer;
  font: inherit;
}
.btn-danger:hover:not(:disabled) {
  filter: brightness(1.1);
}
.btn-danger:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}
.drawer-danger .drawer-title {
  color: var(--color-error);
}
.danger-zone-desc {
  margin: 0 0 0.75rem 0;
  font-size: 0.875rem;
  color: var(--color-text-muted);
}
.progress-panel {
  margin-top: 1rem;
  padding: 1rem;
  background: var(--color-surface, #f5f5f5);
  border-radius: 0.5rem;
  border: 1px solid var(--color-border);
}
.progress-prompt-name {
  font-weight: 600;
  font-size: 0.9375rem;
  margin: 0 0 0.5rem 0;
  color: var(--color-text);
}
.progress-bar-wrap {
  height: 0.5rem;
  background: var(--color-border);
  border-radius: 0.25rem;
  overflow: hidden;
  margin-bottom: 0.5rem;
}
.progress-bar-fill {
  height: 100%;
  background: var(--color-primary);
  border-radius: 0.25rem;
  transition: width 0.2s ease;
}
.progress-count {
  font-size: 0.8125rem;
  color: var(--color-text-muted);
  margin: 0 0 0.75rem 0;
}
.progress-step-message {
  font-size: 0.875rem;
  color: var(--color-text);
  margin: 0;
}
.error {
  color: var(--color-error);
  font-size: 0.875rem;
}
.session-hint {
  margin-top: 0.75rem;
  font-size: 0.875rem;
  color: var(--color-text-muted);
}
.settings-hint {
  font-size: 0.8125rem;
  color: var(--color-text-muted);
  margin: 0 0 0.5rem 0;
}
.config-dir-display {
  font-size: 0.875rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
}
.btn-link {
  background: none;
  border: none;
  color: var(--color-primary);
  cursor: pointer;
  font-size: 0.875rem;
  padding: 0.25rem 0;
  text-decoration: underline;
}
.btn-link:hover {
  color: var(--color-primary-hover);
}
.credentials-check-row {
  margin-top: 0.5rem;
}
.credentials-result {
  margin-top: 0.75rem;
}
.credentials-error {
  margin-bottom: 0.5rem;
  font-size: 0.875rem;
  color: var(--color-error);
}
.credentials-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.875rem;
  background: var(--color-bg);
  border-radius: 0.375rem;
  overflow: hidden;
}
.credentials-table th,
.credentials-table td {
  padding: 0.375rem 0.75rem;
  text-align: left;
  border-bottom: 1px solid var(--color-border);
}
.credentials-table th {
  font-weight: 600;
  color: var(--color-text-muted);
  white-space: nowrap;
  width: 1%;
}
.credentials-table tr:last-child th,
.credentials-table tr:last-child td {
  border-bottom: none;
}
.arn-cell .arn-text {
  word-break: break-all;
}
.form-row-checkbox {
  margin-bottom: 0;
}
.proxy-toggle-row {
  margin-top: 0.75rem;
}
.proxy-fields {
  margin-top: 0.5rem;
}
.proxy-fields input {
  max-width: 28rem;
}
.proxy-test-ok {
  font-size: 0.875rem;
  color: var(--color-success, #0a0);
  margin-top: 0.5rem;
  margin-bottom: 0;
}
.checkbox-label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
  font-weight: 500;
  max-width: 100%;
}
.checkbox-label input {
  margin: 0;
  flex-shrink: 0;
}
.checkbox-label input[type="checkbox"] {
  width: auto;
}
.footer {
  margin-top: 2rem;
  padding-top: 1rem;
  border-top: 1px solid var(--color-border);
  text-align: center;
}
.footer-link {
  background: none;
  border: none;
  color: var(--color-text-muted);
  cursor: pointer;
  font-size: 0.8125rem;
  text-decoration: none;
  padding: 0;
}
.footer-link:hover {
  color: var(--color-primary);
  text-decoration: underline;
}
.about-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  padding: 1rem;
}
.about-panel {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 0.5rem;
  padding: 1.5rem;
  max-width: 28rem;
  max-height: 90vh;
  overflow-y: auto;
}
.about-title {
  font-size: 1.25rem;
  margin: 0 0 0.5rem 0;
}
.about-version {
  font-size: 0.875rem;
  color: var(--color-text-muted);
  margin: 0 0 0.5rem 0;
}
.about-copyright {
  font-size: 0.875rem;
  margin: 0 0 0.75rem 0;
}
.about-license {
  font-size: 0.8125rem;
  color: var(--color-text-muted);
  margin: 0 0 0.5rem 0;
  line-height: 1.5;
}
.about-license a {
  color: var(--color-primary);
}
.about-license a:hover {
  text-decoration: underline;
}
.about-warranty {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  margin: 0 0 0.5rem 0;
}
.about-github {
  font-size: 0.8125rem;
  margin: 0 0 1rem 0;
}
.about-github a {
  color: var(--color-primary);
  text-decoration: none;
}
.about-github a:hover {
  text-decoration: underline;
}
.about-close {
  margin-top: 0.5rem;
}
</style>

const { existsSync } = require("node:fs");
const { createRequire } = require("node:module");
const path = require("node:path");

const requireFromHere = createRequire(__filename);

const PLATFORM_PACKAGES = {
  "win32-x64": { pkg: "@nargo/nargo-win32-x64", dir: "nargo-win32-x64", file: "nargo.win32-x64-msvc.node" },
  "darwin-x64": { pkg: "@nargo/nargo-darwin-x64", dir: "nargo-darwin-x64", file: "nargo.darwin-x64.node" },
  "darwin-arm64": { pkg: "@nargo/nargo-darwin-arm64", dir: "nargo-darwin-arm64", file: "nargo.darwin-arm64.node" },
  "linux-x64": { pkg: "@nargo/nargo-linux-x64", dir: "nargo-linux-x64", file: "nargo.linux-x64-gnu.node" },
  "linux-arm64": { pkg: "@nargo/nargo-linux-arm64", dir: "nargo-linux-arm64", file: "nargo.linux-arm64-gnu.node" },
};

/**
 * Resolve and load the platform NAPI `.node` binding.
 * @returns {{ runCli?: (argv: string[]) => number | void }}
 */
function loadBinding() {
  const key = `${process.platform}-${process.arch}`;
  const meta = PLATFORM_PACKAGES[key];
  if (!meta) {
    throw new Error(`Unsupported platform: ${key}`);
  }

  try {
    return requireFromHere(meta.pkg);
  } catch (first) {
    const local = path.join(__dirname, "..", meta.dir, meta.file);
    if (existsSync(local)) {
      return requireFromHere(local);
    }
    const err = new Error(
      `Failed to load NAPI binding for ${key} (package ${meta.pkg}). Build the native addon first.`,
    );
    err.cause = first;
    throw err;
  }
}

/**
 * @param {string[]} argv
 * @returns {number}
 */
function run(argv = process.argv.slice(2)) {
  const binding = loadBinding();
  if (typeof binding.runCli !== "function") {
    console.error("nargo: native binding loaded but runCli() is not exported yet");
    return 1;
  }
  const code = binding.runCli(argv);
  return typeof code === "number" ? code : 0;
}

module.exports = { run, loadBinding, PLATFORM_PACKAGES };

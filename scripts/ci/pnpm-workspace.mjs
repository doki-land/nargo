/**
 * Discover publishable packages in a pnpm workspace.
 *
 * Reads `<root>/pnpm-workspace.yaml` globs, loads each package.json,
 * skips `private: true`. Returns specs suitable for placeholder / npm publish.
 *
 * @typedef {{ name: string, dir: string, version: string, private: boolean, os?: string[], cpu?: string[], description?: string }} WorkspacePackage
 */

import fs from "node:fs";
import path from "node:path";

/**
 * Minimal YAML list parser for `packages:` globs only.
 * Supports:
 *   packages:
 *     - "runtimes/*"
 *     - documentation
 * @param {string} text
 * @returns {string[]}
 */
export function parsePnpmWorkspacePackages(text) {
    const lines = text.split(/\r?\n/);
    /** @type {string[]} */
    const globs = [];
    let inPackages = false;
    for (const raw of lines) {
        const line = raw.replace(/#.*$/, "").trimEnd();
        if (!line.trim()) continue;
        if (/^packages\s*:/.test(line.trim())) {
            inPackages = true;
            continue;
        }
        if (inPackages) {
            if (/^[A-Za-z_]/.test(line.trim()) && !line.trim().startsWith("-")) {
                inPackages = false;
                continue;
            }
            const m = line.match(/^\s*-\s*(.+)$/);
            if (m) {
                let v = m[1].trim();
                if ((v.startsWith('"') && v.endsWith('"')) || (v.startsWith("'") && v.endsWith("'"))) {
                    v = v.slice(1, -1);
                }
                if (v) globs.push(v);
            }
        }
    }
    return globs;
}

/**
 * Expand a single workspace glob relative to root (supports `*` one segment, `**` recursive dirs).
 * @param {string} root
 * @param {string} glob
 * @returns {string[]} absolute package.json dirs that contain package.json
 */
export function expandWorkspaceGlob(root, glob) {
    const abs = path.resolve(root, glob);
    if (!glob.includes("*")) {
        const pkg = path.join(abs, "package.json");
        return fs.existsSync(pkg) ? [abs] : [];
    }

    /** @type {string[]} */
    const out = [];

    if (glob.endsWith("/*") && !glob.includes("**")) {
        const base = path.resolve(root, glob.slice(0, -2));
        if (!fs.existsSync(base) || !fs.statSync(base).isDirectory()) return [];
        for (const ent of fs.readdirSync(base, { withFileTypes: true })) {
            if (!ent.isDirectory()) continue;
            const dir = path.join(base, ent.name);
            if (fs.existsSync(path.join(dir, "package.json"))) out.push(dir);
        }
        return out;
    }

    // Recursive ** pattern (shallow walk of matching prefix)
    const walk = (dir) => {
        if (!fs.existsSync(dir) || !fs.statSync(dir).isDirectory()) return;
        if (fs.existsSync(path.join(dir, "package.json"))) out.push(dir);
        for (const ent of fs.readdirSync(dir, { withFileTypes: true })) {
            if (!ent.isDirectory()) continue;
            if (ent.name === "node_modules" || ent.name === ".git" || ent.name === "target") continue;
            walk(path.join(dir, ent.name));
        }
    };

    if (glob.includes("**")) {
        const prefix = glob.split("**")[0].replace(/\/$/, "");
        walk(path.resolve(root, prefix || "."));
        return out;
    }

    // Fallback: treat as directory
    if (fs.existsSync(path.join(abs, "package.json"))) out.push(abs);
    return out;
}

/**
 * @param {string} root workspace root (directory containing pnpm-workspace.yaml)
 * @param {{ includePrivate?: boolean }} [opts]
 * @returns {WorkspacePackage[]}
 */
export function discoverPnpmWorkspace(root, opts = {}) {
    const wsFile = path.join(root, "pnpm-workspace.yaml");
    if (!fs.existsSync(wsFile)) {
        throw new Error(`pnpm-workspace.yaml not found under ${root}`);
    }
    const globs = parsePnpmWorkspacePackages(fs.readFileSync(wsFile, "utf8"));
    /** @type {Map<string, WorkspacePackage>} */
    const byName = new Map();

    for (const g of globs) {
        for (const dir of expandWorkspaceGlob(root, g)) {
            const pkgPath = path.join(dir, "package.json");
            const pkg = JSON.parse(fs.readFileSync(pkgPath, "utf8"));
            if (!pkg?.name || typeof pkg.name !== "string") continue;
            if (pkg.private === true && !opts.includePrivate) continue;

            /** @type {WorkspacePackage} */
            const entry = {
                name: pkg.name,
                dir,
                version: typeof pkg.version === "string" ? pkg.version : "0.0.0",
                private: pkg.private === true,
                description: typeof pkg.description === "string" ? pkg.description : undefined,
            };
            if (Array.isArray(pkg.os)) entry.os = pkg.os;
            if (Array.isArray(pkg.cpu)) entry.cpu = pkg.cpu;
            byName.set(pkg.name, entry);
        }
    }

    return [...byName.values()].sort((a, b) => a.name.localeCompare(b.name));
}

/**
 * Topological-ish publish order: packages with no workspace deps first, then dependents.
 * @param {WorkspacePackage[]} packages
 * @returns {WorkspacePackage[]}
 */
export function orderForPublish(packages) {
    const byName = new Map(packages.map((p) => [p.name, p]));
    /** @type {Map<string, Set<string>>} */
    const deps = new Map();

    for (const p of packages) {
        const pkg = JSON.parse(fs.readFileSync(path.join(p.dir, "package.json"), "utf8"));
        /** @type {Set<string>} */
        const set = new Set();
        for (const field of ["dependencies", "optionalDependencies", "peerDependencies"]) {
            const block = pkg[field];
            if (!block || typeof block !== "object") continue;
            for (const name of Object.keys(block)) {
                if (byName.has(name)) set.add(name);
            }
        }
        deps.set(p.name, set);
    }

    /** @type {string[]} */
    const ordered = [];
    /** @type {Set<string>} */
    const visiting = new Set();
    /** @type {Set<string>} */
    const done = new Set();

    /** @param {string} name */
    function visit(name) {
        if (done.has(name)) return;
        if (visiting.has(name)) return; // cycle: ignore
        visiting.add(name);
        for (const d of deps.get(name) ?? []) visit(d);
        visiting.delete(name);
        done.add(name);
        ordered.push(name);
    }

    for (const p of packages) visit(p.name);
    return ordered.map((n) => byName.get(n)).filter(Boolean);
}

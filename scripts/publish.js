const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const ROOT = path.join(__dirname, '..');
const PACKAGES_DIR = path.join(ROOT, 'packages');

function getPackageJson(dir) {
  const p = path.join(dir, 'package.json');
  if (fs.existsSync(p)) {
    return JSON.parse(fs.readFileSync(p, 'utf8'));
  }
  return null;
}

const isDryRun = process.argv.includes('--dry-run');

function publishTSPackages() {
  console.log('--- Building & Publishing TS Packages ---');
  if (isDryRun) {
    console.log('DRY RUN MODE ENABLED - No actual publishing will occur.');
  }
  
  // 0. Build TS Dist files locally before publishing
  console.log('Compiling TypeScript...');
  try {
    execSync('tsc -p packages/tsconfig.json', { cwd: ROOT, stdio: 'inherit' });
  } catch (e) {
    console.error('TypeScript compilation failed!');
    process.exit(1);
  }

  // 1. Sync Rust binaries from artifacts to packages/
  const artifactsDir = path.join(ROOT, 'npm-packages');
  if (fs.existsSync(artifactsDir)) {
    console.log('Syncing Rust binaries from npm-packages to packages/');
    const packages = fs.readdirSync(artifactsDir);
    for (const pkgName of packages) {
      const src = path.join(artifactsDir, pkgName);
      
      // Only handle platform binary packages (nargo-*)
      if (pkgName.startsWith('nargo-') && pkgName !== 'nargo-runtime' && pkgName !== 'nargo-router' && pkgName !== 'nargo-store') {
        const dest = path.join(PACKAGES_DIR, pkgName);
        if (fs.existsSync(src) && fs.statSync(src).isDirectory()) {
          const binDir = path.join(src, 'bin');
          if (fs.existsSync(binDir)) {
            if (!fs.existsSync(path.join(dest, 'bin'))) {
              fs.mkdirSync(path.join(dest, 'bin'), { recursive: true });
            }
            const bins = fs.readdirSync(binDir);
            for (const bin of bins) {
              fs.copyFileSync(path.join(binDir, bin), path.join(dest, 'bin', bin));
              console.log(`Synced ${bin} to ${dest}/bin`);
            }
          }
        }
      }
    }
  }

  // 2. Core Runtimes (Libraries)
  const runtimeLibraries = ['nargo-runtime', 'nargo-store', 'nargo-router'];
  for (const name of runtimeLibraries) {
    const dir = path.join(PACKAGES_DIR, name);
    if (fs.existsSync(dir)) {
      console.log(`${isDryRun ? 'Dry-run: ' : ''}Publishing runtime library ${name}...`);
      try {
        const cmd = isDryRun ? 'npm publish --access public --dry-run' : 'npm publish --access public';
        execSync(cmd, { cwd: dir, stdio: 'inherit' });
      } catch (e) {
        console.warn(`Failed to publish ${name}.`);
      }
    }
  }

  // 3. Binary Distribution Packages
  const platformPkgs = [
    'nargo-darwin-x64', 'nargo-darwin-arm64',
    'nargo-linux-x64', 'nargo-linux-arm64',
    'nargo-win32-x64'
  ]; // dirs; package names are @nargo/nargo-<platform>
  for (const name of platformPkgs) {
    const dir = path.join(PACKAGES_DIR, name);
    if (fs.existsSync(dir)) {
      console.log(`${isDryRun ? 'Dry-run: ' : ''}Publishing platform package ${name}...`);
      try {
        const cmd = isDryRun ? 'npm publish --access public --dry-run' : 'npm publish --access public';
        execSync(cmd, { cwd: dir, stdio: 'inherit' });
      } catch (e) {
        console.warn(`Failed to publish ${name}.`);
      }
    }
  }

  // 4. Main Nargo Package (CLI & JSR)
  const mainPkgDir = path.join(PACKAGES_DIR, 'nargo');
  if (fs.existsSync(mainPkgDir)) {
    console.log(`${isDryRun ? 'Dry-run: ' : ''}Publishing main nargo package to NPM...`);
    try {
      const cmd = isDryRun ? 'npm publish --access public --dry-run' : 'npm publish --access public';
      execSync(cmd, { cwd: mainPkgDir, stdio: 'inherit' });
    } catch (e) {
      console.warn(`Failed to publish nargo to NPM.`);
    }

    console.log(`${isDryRun ? 'Dry-run: ' : ''}Publishing main nargo package to JSR...`);
    try {
      const cmd = isDryRun ? 'npx jsr publish --dry-run' : 'npx jsr publish';
      execSync(cmd, { cwd: mainPkgDir, stdio: 'inherit' });
    } catch (e) { 
      console.warn(`Failed to publish nargo to JSR.`);
    }
  }
}

function publishRustCrates() {
  console.log('--- Publishing Rust Crates ---');
  if (isDryRun) {
    console.log('DRY RUN MODE ENABLED - No actual publishing will occur.');
  }

  // Rust publishing usually handled by cargo-release or manual order
  // For Nargo, the order is: types -> [others] -> nargo
  const crates = [
    'projects/nargo-types',
    'projects/nargo-parser',
    'projects/nargo-compiler',
    'projects/nargo-bundler',
    'projects/nargo-server',
    'projects/nargo-linter',
    'projects/nargo-test-runner',
    'projects/nargo-coverage',
    'projects/nargo-bench',
    'projects/nargo-audit',
    'projects/nargo-analyzer',
    'projects/nargo-mono',
    'projects/nargo-mock',
    'projects/nargo-optimizer',
    'projects/nargo-i18n',
    'projects/nargo-hooks',
    'projects/nargo-bridge',
    'projects/nargo-release',
    'projects/nargo-scaffolder',
    'projects/nargo'
  ];

  for (const relPath of crates) {
    const dir = path.join(ROOT, relPath);
    console.log(`${isDryRun ? 'Dry-run: ' : ''}Publishing Rust crate in ${relPath}...`);
    try {
      const cmd = isDryRun ? 'cargo publish --dry-run' : 'cargo publish';
      execSync(cmd, { cwd: dir, stdio: 'inherit' });
    } catch (e) {
      console.warn(`Failed to publish ${relPath}, it might already exist or need login.`);
    }
  }
}

// Main execution
const mode = process.argv[2] || 'all';

if (mode === 'ts' || mode === 'all') {
  publishTSPackages();
}

if (mode === 'rust' || mode === 'all') {
  publishRustCrates();
}

import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { createRequire } from 'node:module';
import { writeFileSync } from 'node:fs';
import path from 'node:path';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const require = createRequire(import.meta.url);
const napiPkg = require('@napi-rs/cli/package.json');
const napiCli = path.resolve(path.dirname(require.resolve('@napi-rs/cli/package.json')), typeof napiPkg.bin === 'string' ? napiPkg.bin : napiPkg.bin.napi);
const release = process.argv.slice(2).includes('--release');

function build() {
  const args = ['build', '--platform', '--dts', 'index.d.ts', '--js', 'index.cjs'];
  if (release) args.push('--release');
  if (process.env.NAPI_TARGET) args.push('--target', process.env.NAPI_TARGET);
  args.push('--cargo-flags=--locked');
  execFileSync(process.execPath, [napiCli, ...args], { cwd: root, stdio: 'inherit' });

  const esmContent = `import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const nativeBinding = require('./index.cjs');

export const {
  Language,
  Status,
  CommentKind,
  parseSync,
  parse,
  detectLanguage,
  supportedLanguages,
} = nativeBinding;

export default nativeBinding;
`;
  writeFileSync(path.join(root, 'index.js'), esmContent, 'utf8');
}

build();

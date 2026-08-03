import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const release = process.argv.slice(2).includes('--release');
const napiCli = path.join(root, 'node_modules', '@napi-rs', 'cli', 'dist', 'cli.js');

function build(jsFile, esm) {
  const args = ['build', '--platform'];
  if (esm) args.push('--esm');
  args.push('--js-package-name', '@astraive/ripex', '--js', jsFile);
  if (release) args.push('--release');
  if (process.env.NAPI_TARGET) args.push('--target', process.env.NAPI_TARGET);
  if (process.env.NAPI_CROSS === '1') args.push('--use-napi-cross');
  args.push('--no-const-enum', '--no-dts-cache', '--', '--locked');
  execFileSync(process.execPath, [napiCli, ...args], { cwd: root, stdio: 'inherit' });
}

build('index.cjs', false);
build('index.js', true);

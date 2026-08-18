import { createRequire } from 'node:module';

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

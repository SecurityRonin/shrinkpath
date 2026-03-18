import { defineConfig } from 'tsup';

export default defineConfig({
  entry: {
    index: 'src/index.ts',
    'fs-aware': 'src/fs-aware.ts',
  },
  format: ['esm', 'cjs'],
  dts: true,
  clean: true,
  target: ['node18', 'es2022'],
  splitting: false,
  external: ['node:fs', 'node:path'],
});

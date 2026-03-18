import { describe, it, expect } from 'bun:test';
import { getExportedSymbols } from '../src/exports.js';
import { writeFileSync, mkdirSync, rmSync } from 'node:fs';
import { join } from 'node:path';

const TMP = join(import.meta.dir, '.tmp');

describe('getExportedSymbols', () => {
  it('extracts function exports', () => {
    mkdirSync(TMP, { recursive: true });
    const file = join(TMP, 'funcs.ts');
    writeFileSync(file, `
export function greet(name: string): string { return name; }
export async function fetchData(): Promise<void> {}
export const MAX = 100;
export type Config = { key: string };
export interface Options { verbose: boolean }
export class Service {}
export enum Status { Active, Inactive }
`);
    const symbols = getExportedSymbols(file);
    expect(symbols).toContain('greet');
    expect(symbols).toContain('fetchData');
    expect(symbols).toContain('MAX');
    expect(symbols).toContain('Config');
    expect(symbols).toContain('Options');
    expect(symbols).toContain('Service');
    expect(symbols).toContain('Status');
    rmSync(TMP, { recursive: true });
  });

  it('extracts re-exports', () => {
    mkdirSync(TMP, { recursive: true });
    const file = join(TMP, 'reexport.ts');
    writeFileSync(file, `
export { foo, bar as baz } from './other.js';
export type { MyType } from './types.js';
`);
    const symbols = getExportedSymbols(file);
    expect(symbols).toContain('baz');
    expect(symbols).toContain('MyType');
    rmSync(TMP, { recursive: true });
  });

  it('returns empty for non-existent file', () => {
    expect(getExportedSymbols('/tmp/does-not-exist.ts')).toEqual([]);
  });
});

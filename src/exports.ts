import { existsSync, readFileSync } from 'node:fs';

/** Strip single-line and multi-line comments from TypeScript source */
function stripComments(src: string): string {
  return src.replace(/\/\/.*$/gm, '').replace(/\/\*[\s\S]*?\*\//g, '');
}

/** Extract exported symbol names from a TypeScript file */
export function getExportedSymbols(filePath: string): string[] {
  if (!existsSync(filePath)) return [];
  const content = stripComments(readFileSync(filePath, 'utf-8'));
  const symbols: string[] = [];

  // export function/class/interface/type/const/enum name
  const regex = /export\s+(?:async\s+)?(?:abstract\s+)?(?:function|class|interface|type|const|enum)\s+(\w+)/g;
  let match: RegExpExecArray | null;
  while ((match = regex.exec(content)) !== null) {
    symbols.push(match[1]);
  }

  // export type { Name } (re-exports)
  const reExportRegex = /export\s+type\s*\{\s*([^}]+)\}/g;
  while ((match = reExportRegex.exec(content)) !== null) {
    const names = match[1].split(',').map((n) => n.trim().split(/\s+as\s+/).pop()!.trim());
    symbols.push(...names.filter(Boolean));
  }

  // export { Name } (re-exports)
  const reExportRegex2 = /export\s*\{\s*([^}]+)\}/g;
  while ((match = reExportRegex2.exec(content)) !== null) {
    const full = match[0];
    if (full.includes('export type')) continue;
    const names = match[1].split(',').map((n) => {
      let name = n.trim().split(/\s+as\s+/).pop()!.trim();
      if (name.startsWith('type ')) name = name.slice(5).trim();
      return name;
    });
    symbols.push(...names.filter(Boolean));
  }

  return [...new Set(symbols)];
}

import * as vscode from "vscode";
import { execFile } from "child_process";
import { promisify } from "util";
import * as path from "path";

const execFileAsync = promisify(execFile);

// ── Types matching CLI JSON output ──────────────────────────────────────

interface CheckOutput {
  passed: boolean;
  specs_checked: number;
  errors: string[];
  warnings: string[];
}

interface CoverageOutput {
  file_coverage: number;
  files_covered: number;
  files_total: number;
  loc_coverage: number;
  loc_covered: number;
  loc_total: number;
  modules: { name: string; has_spec: boolean }[];
  uncovered_files: { file: string; loc: number }[];
}

interface ScoreSpec {
  spec: string;
  total: number;
  grade: string;
  frontmatter: number;
  sections: number;
  api: number;
  depth: number;
  freshness: number;
  suggestions: string[];
}

interface ScoreOutput {
  average_score: number;
  grade: string;
  total_specs: number;
  distribution: { A: number; B: number; C: number; D: number; F: number };
  specs: ScoreSpec[];
}

interface GenerateOutput {
  generated: string[];
}

// ── Global state ────────────────────────────────────────────────────────

let diagnosticCollection: vscode.DiagnosticCollection;
let statusBarItem: vscode.StatusBarItem;
let outputChannel: vscode.OutputChannel;
let checkDebounceTimer: ReturnType<typeof setTimeout> | undefined;
let lastScoreResult: ScoreOutput | undefined;

// ── Activation ──────────────────────────────────────────────────────────

export function activate(context: vscode.ExtensionContext) {
  outputChannel = vscode.window.createOutputChannel("SpecSync");
  diagnosticCollection = vscode.languages.createDiagnosticCollection("specsync");

  statusBarItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Left,
    50
  );
  statusBarItem.command = "specsync.check";
  statusBarItem.tooltip = "SpecSync — click to validate";
  statusBarItem.show();

  // Register commands
  context.subscriptions.push(
    vscode.commands.registerCommand("specsync.check", () => runCheck()),
    vscode.commands.registerCommand("specsync.coverage", () => runCoverage(context)),
    vscode.commands.registerCommand("specsync.score", () => runScore(context)),
    vscode.commands.registerCommand("specsync.generate", () => runGenerate()),
    vscode.commands.registerCommand("specsync.init", () => runInit()),
    diagnosticCollection,
    statusBarItem,
    outputChannel
  );

  // CodeLens for inline scores
  const codeLensProvider = new SpecScoreCodeLensProvider();
  context.subscriptions.push(
    vscode.languages.registerCodeLensProvider(
      { pattern: "**/*.spec.md" },
      codeLensProvider
    )
  );

  // Validate on save (debounced)
  context.subscriptions.push(
    vscode.workspace.onDidSaveTextDocument((doc) => {
      const config = vscode.workspace.getConfiguration("specsync");
      if (!config.get<boolean>("validateOnSave", true)) {
        return;
      }
      if (doc.fileName.endsWith(".spec.md") || isSourceFile(doc.fileName)) {
        debouncedCheck();
      }
    })
  );

  // Re-read config on change
  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration((e) => {
      if (e.affectsConfiguration("specsync")) {
        log("Configuration changed, re-running check");
        runCheck();
      }
    })
  );

  // Initial check
  setStatusBar("syncing", "$(sync~spin) SpecSync");
  runCheck();
}

export function deactivate() {
  diagnosticCollection?.clear();
  if (checkDebounceTimer) {
    clearTimeout(checkDebounceTimer);
  }
}

// ── CLI runner ──────────────────────────────────────────────────────────

function getBinary(): string {
  return (
    vscode.workspace
      .getConfiguration("specsync")
      .get<string>("binaryPath") ?? "specsync"
  );
}

function getRoot(): string {
  return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath ?? ".";
}

async function runSpecsyncRaw(args: string[]): Promise<string> {
  const binary = getBinary();
  const root = getRoot();
  const fullArgs = [...args, "--root", root];
  log(`> ${binary} ${fullArgs.join(" ")}`);

  try {
    const { stdout, stderr } = await execFileAsync(binary, fullArgs, {
      cwd: root,
      timeout: 30_000,
    });
    if (stderr) {
      log(`stderr: ${stderr}`);
    }
    return stdout;
  } catch (err: unknown) {
    const execErr = err as { stdout?: string; stderr?: string; code?: number };
    if (execErr.stdout) {
      if (execErr.stderr) {
        log(`stderr: ${execErr.stderr}`);
      }
      return execErr.stdout;
    }
    throw err;
  }
}

async function runSpecsync(args: string[]): Promise<string> {
  return runSpecsyncRaw([...args, "--json"]);
}

// ── Logging ─────────────────────────────────────────────────────────────

function log(message: string) {
  const ts = new Date().toLocaleTimeString();
  outputChannel.appendLine(`[${ts}] ${message}`);
}

// ── Status bar ──────────────────────────────────────────────────────────

function setStatusBar(
  state: "pass" | "fail" | "syncing" | "error",
  text: string
) {
  statusBarItem.text = text;
  switch (state) {
    case "pass":
      statusBarItem.backgroundColor = undefined;
      break;
    case "fail":
      statusBarItem.backgroundColor = new vscode.ThemeColor(
        "statusBarItem.warningBackground"
      );
      break;
    case "error":
      statusBarItem.backgroundColor = new vscode.ThemeColor(
        "statusBarItem.errorBackground"
      );
      break;
    case "syncing":
      statusBarItem.backgroundColor = undefined;
      break;
  }
}

// ── Debounced check ─────────────────────────────────────────────────────

function debouncedCheck() {
  if (checkDebounceTimer) {
    clearTimeout(checkDebounceTimer);
  }
  checkDebounceTimer = setTimeout(() => {
    runCheck();
  }, 500);
}

// ── Commands ────────────────────────────────────────────────────────────

async function runCheck() {
  setStatusBar("syncing", "$(sync~spin) SpecSync");

  try {
    const stdout = await runSpecsync(["check"]);
    const result: CheckOutput = JSON.parse(stdout);

    // Invalidate score cache so CodeLens refreshes on next view
    lastScoreResult = undefined;

    diagnosticCollection.clear();
    const diagnosticMap = new Map<vscode.Uri, vscode.Diagnostic[]>();

    for (const error of result.errors ?? []) {
      const { uri, diagnostic } = parseDiagnostic(error, vscode.DiagnosticSeverity.Error);
      const existing = diagnosticMap.get(uri) ?? [];
      existing.push(diagnostic);
      diagnosticMap.set(uri, existing);
    }

    for (const warning of result.warnings ?? []) {
      const { uri, diagnostic } = parseDiagnostic(warning, vscode.DiagnosticSeverity.Warning);
      const existing = diagnosticMap.get(uri) ?? [];
      existing.push(diagnostic);
      diagnosticMap.set(uri, existing);
    }

    for (const [uri, diagnostics] of diagnosticMap) {
      diagnosticCollection.set(uri, diagnostics);
    }

    const errorCount = result.errors?.length ?? 0;
    const warnCount = result.warnings?.length ?? 0;

    if (result.passed) {
      setStatusBar("pass", `$(check) SpecSync: ${result.specs_checked} specs OK`);
    } else {
      setStatusBar("fail", `$(warning) SpecSync: ${errorCount}E ${warnCount}W`);
    }

    log(`Check complete: ${result.specs_checked} specs, ${errorCount} errors, ${warnCount} warnings`);
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    setStatusBar("error", "$(error) SpecSync");
    log(`Check failed: ${msg}`);
    vscode.window.showErrorMessage(`SpecSync: ${msg}`);
  }
}

async function runCoverage(context: vscode.ExtensionContext) {
  try {
    const stdout = await runSpecsync(["coverage"]);
    const result: CoverageOutput = JSON.parse(stdout);

    const panel = vscode.window.createWebviewPanel(
      "specsyncCoverage",
      "SpecSync Coverage",
      vscode.ViewColumn.One,
      { enableScripts: false }
    );

    const uncoveredRows = result.uncovered_files
      .sort((a, b) => b.loc - a.loc)
      .map(
        (f) =>
          `<tr><td>${escapeHtml(f.file)}</td><td class="num">${f.loc}</td></tr>`
      )
      .join("\n");

    const moduleRows = result.modules
      .filter((m) => !m.has_spec)
      .map((m) => `<li>${escapeHtml(m.name)}</li>`)
      .join("\n");

    const filePct = result.file_coverage.toFixed(1);
    const locPct = result.loc_coverage.toFixed(1);

    panel.webview.html = webviewHtml(
      "SpecSync Coverage",
      `
      <div class="stats">
        <div class="stat">
          <span class="stat-value">${filePct}%</span>
          <span class="stat-label">File coverage (${result.files_covered}/${result.files_total})</span>
        </div>
        <div class="stat">
          <span class="stat-value">${locPct}%</span>
          <span class="stat-label">LOC coverage (${result.loc_covered}/${result.loc_total})</span>
        </div>
      </div>

      ${
        result.uncovered_files.length > 0
          ? `<h2>Uncovered Files</h2>
             <table>
               <thead><tr><th>File</th><th class="num">LOC</th></tr></thead>
               <tbody>${uncoveredRows}</tbody>
             </table>`
          : "<p>All files covered!</p>"
      }

      ${
        moduleRows
          ? `<h2>Unspecced Modules</h2><ul>${moduleRows}</ul>`
          : ""
      }
      `
    );

    log(`Coverage: ${filePct}% files, ${locPct}% LOC`);
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    log(`Coverage failed: ${msg}`);
    vscode.window.showErrorMessage(`SpecSync coverage: ${msg}`);
  }
}

async function runScore(context: vscode.ExtensionContext) {
  try {
    const stdout = await runSpecsync(["score"]);
    const result: ScoreOutput = JSON.parse(stdout);
    lastScoreResult = result;

    const panel = vscode.window.createWebviewPanel(
      "specsyncScore",
      "SpecSync Quality Scores",
      vscode.ViewColumn.One,
      { enableScripts: false }
    );

    const specRows = result.specs
      .sort((a, b) => a.total - b.total)
      .map(
        (s) => `
        <tr>
          <td>${escapeHtml(s.spec)}</td>
          <td class="grade grade-${s.grade}">${s.grade}</td>
          <td class="num">${s.total}</td>
          <td class="num">${s.frontmatter}</td>
          <td class="num">${s.sections}</td>
          <td class="num">${s.api}</td>
          <td class="num">${s.depth}</td>
          <td class="num">${s.freshness}</td>
          <td>${s.suggestions.length > 0 ? s.suggestions.map(escapeHtml).join("<br>") : '<span class="muted">—</span>'}</td>
        </tr>`
      )
      .join("\n");

    const dist = result.distribution;

    panel.webview.html = webviewHtml(
      "SpecSync Quality Scores",
      `
      <div class="stats">
        <div class="stat">
          <span class="stat-value grade-${result.grade}">${result.grade}</span>
          <span class="stat-label">Overall grade</span>
        </div>
        <div class="stat">
          <span class="stat-value">${result.average_score}</span>
          <span class="stat-label">Average score</span>
        </div>
        <div class="stat">
          <span class="stat-value">${result.total_specs}</span>
          <span class="stat-label">Total specs</span>
        </div>
      </div>

      <h2>Distribution</h2>
      <div class="dist">
        <span class="dist-bar"><span class="grade-A">A</span>: ${dist.A}</span>
        <span class="dist-bar"><span class="grade-B">B</span>: ${dist.B}</span>
        <span class="dist-bar"><span class="grade-C">C</span>: ${dist.C}</span>
        <span class="dist-bar"><span class="grade-D">D</span>: ${dist.D}</span>
        <span class="dist-bar"><span class="grade-F">F</span>: ${dist.F}</span>
      </div>

      <h2>Spec Details</h2>
      <table>
        <thead>
          <tr>
            <th>Spec</th><th>Grade</th><th>Score</th>
            <th>FM</th><th>Sec</th><th>API</th><th>Depth</th><th>Fresh</th>
            <th>Suggestions</th>
          </tr>
        </thead>
        <tbody>${specRows}</tbody>
      </table>
      `
    );

    log(`Score: ${result.average_score}/100 [${result.grade}], ${result.total_specs} specs`);
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    log(`Score failed: ${msg}`);
    vscode.window.showErrorMessage(`SpecSync score: ${msg}`);
  }
}

async function runGenerate() {
  try {
    const stdout = await runSpecsync(["generate"]);
    const result: GenerateOutput = JSON.parse(stdout);
    const count = result.generated?.length ?? 0;

    if (count === 0) {
      vscode.window.showInformationMessage("SpecSync: All modules already have specs");
    } else {
      vscode.window.showInformationMessage(
        `SpecSync: Generated ${count} spec file(s)`
      );
      // Re-run check to pick up new specs
      runCheck();
    }

    log(`Generate: ${count} specs created`);
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    log(`Generate failed: ${msg}`);
    vscode.window.showErrorMessage(`SpecSync generate: ${msg}`);
  }
}

async function runInit() {
  try {
    await runSpecsyncRaw(["init"]);
    vscode.window.showInformationMessage("SpecSync: Created specsync.json");
    log("Init: created specsync.json");
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    if (msg.includes("already exists")) {
      vscode.window.showInformationMessage("SpecSync: specsync.json already exists");
    } else {
      log(`Init failed: ${msg}`);
      vscode.window.showErrorMessage(`SpecSync init: ${msg}`);
    }
  }
}

// ── CodeLens provider ───────────────────────────────────────────────────

class SpecScoreCodeLensProvider implements vscode.CodeLensProvider {
  private _onDidChangeCodeLenses = new vscode.EventEmitter<void>();
  readonly onDidChangeCodeLenses = this._onDidChangeCodeLenses.event;

  async provideCodeLenses(
    document: vscode.TextDocument
  ): Promise<vscode.CodeLens[]> {
    const config = vscode.workspace.getConfiguration("specsync");
    if (!config.get<boolean>("showInlineScores", true)) {
      return [];
    }

    // Fetch scores if we don't have them cached
    if (!lastScoreResult) {
      try {
        const stdout = await runSpecsync(["score"]);
        lastScoreResult = JSON.parse(stdout);
      } catch {
        return [];
      }
    }

    const root = getRoot();
    const relPath = path.relative(root, document.uri.fsPath);

    const spec = lastScoreResult?.specs.find(
      (s) => s.spec === relPath || s.spec === relPath.replace(/\\/g, "/")
    );

    if (!spec) {
      return [];
    }

    const range = new vscode.Range(0, 0, 0, 0);

    const lenses: vscode.CodeLens[] = [
      new vscode.CodeLens(range, {
        title: `$(star) ${spec.grade} (${spec.total}/100) — FM:${spec.frontmatter} Sec:${spec.sections} API:${spec.api} Depth:${spec.depth} Fresh:${spec.freshness}`,
        command: "specsync.score",
        tooltip: "Open full quality report",
      }),
    ];

    if (spec.suggestions.length > 0) {
      lenses.push(
        new vscode.CodeLens(range, {
          title: `$(lightbulb) ${spec.suggestions[0]}`,
          command: "",
          tooltip: spec.suggestions.join("\n"),
        })
      );
    }

    return lenses;
  }
}

// ── Helpers ─────────────────────────────────────────────────────────────

function parseDiagnostic(
  message: string,
  severity: vscode.DiagnosticSeverity
): { uri: vscode.Uri; diagnostic: vscode.Diagnostic } {
  // Messages from the CLI look like "specs/foo.spec.md: missing section: Purpose"
  // or just plain "missing section: Purpose"
  const root = getRoot();
  let specPath = "";
  let text = message;

  // CLI prefixes errors with "path/to/spec.spec.md: message"
  const colonMatch = message.match(/^([^:]+\.spec\.md):\s*(.+)$/);
  if (colonMatch) {
    specPath = colonMatch[1];
    text = colonMatch[2];
  }

  const uri = specPath
    ? vscode.Uri.file(path.join(root, specPath))
    : vscode.Uri.file(path.join(root, "specs"));

  const diagnostic = new vscode.Diagnostic(
    new vscode.Range(0, 0, 0, 0),
    text,
    severity
  );
  diagnostic.source = "specsync";
  return { uri, diagnostic };
}

function isSourceFile(filename: string): boolean {
  const exts = [
    ".ts", ".tsx", ".js", ".jsx", ".mts", ".cts",
    ".rs", ".go", ".py", ".swift",
    ".kt", ".kts", ".java", ".cs", ".dart",
    ".php", ".rb",
  ];
  return exts.some((ext) => filename.endsWith(ext));
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function webviewHtml(title: string, body: string): string {
  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>${escapeHtml(title)}</title>
  <style>
    body {
      font-family: var(--vscode-font-family);
      font-size: var(--vscode-font-size);
      color: var(--vscode-foreground);
      background: var(--vscode-editor-background);
      padding: 16px 24px;
      line-height: 1.5;
    }
    h1 {
      font-size: 1.4em;
      margin: 0 0 16px;
      border-bottom: 1px solid var(--vscode-panel-border);
      padding-bottom: 8px;
    }
    h2 {
      font-size: 1.1em;
      margin: 24px 0 8px;
    }
    .stats {
      display: flex;
      gap: 32px;
      margin: 16px 0;
    }
    .stat {
      display: flex;
      flex-direction: column;
      align-items: center;
    }
    .stat-value {
      font-size: 2em;
      font-weight: bold;
      color: var(--vscode-textLink-foreground);
    }
    .stat-label {
      font-size: 0.85em;
      color: var(--vscode-descriptionForeground);
    }
    table {
      width: 100%;
      border-collapse: collapse;
      margin: 8px 0;
    }
    th, td {
      text-align: left;
      padding: 6px 10px;
      border-bottom: 1px solid var(--vscode-panel-border);
    }
    th {
      font-weight: 600;
      color: var(--vscode-descriptionForeground);
      font-size: 0.85em;
      text-transform: uppercase;
    }
    .num { text-align: right; font-variant-numeric: tabular-nums; }
    .grade { font-weight: bold; text-align: center; }
    .grade-A { color: #4ec94e; }
    .grade-B { color: #8bc34a; }
    .grade-C { color: #ffc107; }
    .grade-D { color: #ff9800; }
    .grade-F { color: #f44336; }
    .muted { color: var(--vscode-descriptionForeground); }
    .dist { display: flex; gap: 16px; margin: 8px 0; }
    .dist-bar { font-variant-numeric: tabular-nums; }
    ul { padding-left: 20px; }
    li { margin: 2px 0; }
  </style>
</head>
<body>
  <h1>${escapeHtml(title)}</h1>
  ${body}
</body>
</html>`;
}

import * as vscode from "vscode";
import { execFile } from "child_process";
import { promisify } from "util";

const execFileAsync = promisify(execFile);

const diagnosticCollection =
  vscode.languages.createDiagnosticCollection("specsync");

interface SpecResult {
  spec: string;
  passed: boolean;
  errors: string[];
  warnings: string[];
}

interface CheckOutput {
  passed: boolean;
  specs_checked: number;
  specs: SpecResult[];
}

interface ScoreResult {
  spec: string;
  total: number;
  grade: string;
  suggestions: string[];
}

interface ScoreOutput {
  average_score: number;
  grade: string;
  specs: ScoreResult[];
}

export function activate(context: vscode.ExtensionContext) {
  const config = vscode.workspace.getConfiguration("specsync");

  // Register commands
  context.subscriptions.push(
    vscode.commands.registerCommand("specsync.check", () => runCheck()),
    vscode.commands.registerCommand("specsync.coverage", () => runCoverage()),
    vscode.commands.registerCommand("specsync.score", () => runScore()),
    vscode.commands.registerCommand("specsync.generate", () => runGenerate()),
    vscode.commands.registerCommand("specsync.init", () => runInit()),
    diagnosticCollection
  );

  // Validate on save if enabled
  if (config.get<boolean>("validateOnSave", true)) {
    context.subscriptions.push(
      vscode.workspace.onDidSaveTextDocument((doc) => {
        if (
          doc.fileName.endsWith(".spec.md") ||
          isSourceFile(doc.fileName)
        ) {
          runCheck();
        }
      })
    );
  }

  // Initial check
  runCheck();
}

export function deactivate() {
  diagnosticCollection.clear();
}

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

async function runSpecsync(
  args: string[]
): Promise<{ stdout: string; stderr: string }> {
  const binary = getBinary();
  const root = getRoot();
  return execFileAsync(binary, [...args, "--root", root, "--json"]);
}

async function runCheck() {
  try {
    const { stdout } = await runSpecsync(["check"]);
    const result: CheckOutput = JSON.parse(stdout);

    diagnosticCollection.clear();
    const diagnosticMap = new Map<string, vscode.Diagnostic[]>();

    for (const spec of result.specs ?? []) {
      const uri = vscode.Uri.file(`${getRoot()}/${spec.spec}`);
      const diagnostics: vscode.Diagnostic[] = [];

      for (const error of spec.errors) {
        diagnostics.push(
          new vscode.Diagnostic(
            new vscode.Range(0, 0, 0, 0),
            error,
            vscode.DiagnosticSeverity.Error
          )
        );
      }
      for (const warning of spec.warnings) {
        diagnostics.push(
          new vscode.Diagnostic(
            new vscode.Range(0, 0, 0, 0),
            warning,
            vscode.DiagnosticSeverity.Warning
          )
        );
      }

      diagnosticMap.set(uri.toString(), diagnostics);
    }

    for (const [uri, diagnostics] of diagnosticMap) {
      diagnosticCollection.set(vscode.Uri.parse(uri), diagnostics);
    }

    const icon = result.passed ? "$(check)" : "$(error)";
    vscode.window.setStatusBarMessage(
      `${icon} SpecSync: ${result.specs_checked} specs checked`,
      5000
    );
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    vscode.window.showErrorMessage(`SpecSync check failed: ${msg}`);
  }
}

async function runCoverage() {
  try {
    const { stdout } = await runSpecsync(["coverage"]);
    const result = JSON.parse(stdout);
    const panel = vscode.window.createWebviewPanel(
      "specsyncCoverage",
      "SpecSync Coverage",
      vscode.ViewColumn.One,
      {}
    );
    panel.webview.html = `
      <h1>SpecSync Coverage</h1>
      <p>File coverage: ${result.files_covered}/${result.files_total} (${result.file_coverage}%)</p>
      <p>LOC coverage: ${result.loc_covered}/${result.loc_total} (${result.loc_coverage}%)</p>
      <h2>Uncovered Files</h2>
      <ul>
        ${(result.uncovered_files ?? []).map((f: { file: string; loc: number }) => `<li>${f.file} (${f.loc} LOC)</li>`).join("")}
      </ul>
    `;
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    vscode.window.showErrorMessage(`SpecSync coverage failed: ${msg}`);
  }
}

async function runScore() {
  try {
    const { stdout } = await runSpecsync(["score"]);
    const result: ScoreOutput = JSON.parse(stdout);
    const panel = vscode.window.createWebviewPanel(
      "specsyncScore",
      "SpecSync Quality Scores",
      vscode.ViewColumn.One,
      {}
    );

    const specRows = result.specs
      .map(
        (s) =>
          `<tr>
        <td>${s.spec}</td>
        <td><strong>${s.grade}</strong></td>
        <td>${s.total}/100</td>
        <td>${s.suggestions.length > 0 ? s.suggestions.join("<br>") : "None"}</td>
      </tr>`
      )
      .join("");

    panel.webview.html = `
      <h1>SpecSync Quality Scores</h1>
      <p>Average: <strong>${result.average_score}/100 [${result.grade}]</strong></p>
      <table border="1" cellpadding="6">
        <tr><th>Spec</th><th>Grade</th><th>Score</th><th>Suggestions</th></tr>
        ${specRows}
      </table>
    `;
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    vscode.window.showErrorMessage(`SpecSync score failed: ${msg}`);
  }
}

async function runGenerate() {
  try {
    const { stdout } = await runSpecsync(["generate"]);
    const result = JSON.parse(stdout);
    const count = result.generated?.length ?? 0;
    vscode.window.showInformationMessage(
      `SpecSync: Generated ${count} spec file(s)`
    );
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    vscode.window.showErrorMessage(`SpecSync generate failed: ${msg}`);
  }
}

async function runInit() {
  try {
    const { stdout } = await runSpecsync(["init"]);
    const result = JSON.parse(stdout);
    vscode.window.showInformationMessage(
      result.created
        ? "SpecSync: Created specsync.json"
        : "SpecSync: specsync.json already exists"
    );
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    vscode.window.showErrorMessage(`SpecSync init failed: ${msg}`);
  }
}

function isSourceFile(filename: string): boolean {
  const exts = [
    ".ts",
    ".tsx",
    ".js",
    ".jsx",
    ".rs",
    ".go",
    ".py",
    ".swift",
    ".kt",
    ".java",
    ".cs",
    ".dart",
  ];
  return exts.some((ext) => filename.endsWith(ext));
}

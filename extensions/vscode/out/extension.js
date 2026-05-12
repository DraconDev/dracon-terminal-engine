"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = __importStar(require("vscode"));
const path = __importStar(require("path"));
const cp = __importStar(require("child_process"));
const events_1 = require("events");
const THEME_COLORS = {
    nord: {
        primary: '#81A1C1',
        background: '#2E3440',
        foreground: '#D8DEE9',
        cursor: '#D8DEE9'
    },
    dracula: {
        primary: '#BD93F9',
        background: '#282A36',
        foreground: '#F8F8F2',
        cursor: '#F8F8F2'
    },
    monokai: {
        primary: '#F92672',
        background: '#272822',
        foreground: '#F8F8F2',
        cursor: '#F8F8F2'
    },
    solarized: {
        primary: '#268BD2',
        background: '#002B36',
        foreground: '#839496',
        cursor: '#839496'
    },
    gruvbox: {
        primary: '#83A598',
        background: '#282828',
        foreground: '#EBDBB2',
        cursor: '#EBDBB2'
    },
    cyberpunk: {
        primary: '#FF0080',
        background: '#0D0221',
        foreground: '#FFFFFF',
        cursor: '#00FF9F'
    }
};
// ============================================================================
// TextDocumentContentProvider
// ============================================================================
class DraconTUIContentProvider {
    constructor() {
        this._emitter = new events_1.EventEmitter();
        this._changeEvent = new vscode.EventEmitter();
        this._state = {
            process: null,
            output: '',
            uri: vscode.Uri.parse('dracon-tui://preview/output'),
            panel: null,
            document: null
        };
    }
    get uri() {
        return this._state.uri;
    }
    get state() {
        return this._state;
    }
    updateOutput(output) {
        this._state.output = output;
        this._changeEvent.fire(this._state.uri);
    }
    appendOutput(chunk) {
        this._state.output += chunk;
        this._changeEvent.fire(this._state.uri);
    }
    clearOutput() {
        this._state.output = '';
        this._changeEvent.fire(this._state.uri);
    }
    provideTextDocumentContent(_uri) {
        const theme = vscode.workspace.getConfiguration('dracon').get('theme', 'nord');
        const themeConfig = THEME_COLORS[theme] || THEME_COLORS.nord;
        return this.generateHtml(this._state.output, themeConfig);
    }
    generateHtml(output, theme) {
        // Convert ANSI-like output to HTML with proper styling
        const styledOutput = this.convertToStyledHtml(output, theme);
        return `<!DOCTYPE html>
<html>
<head>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        body {
            background-color: ${theme.background};
            color: ${theme.foreground};
            font-family: 'Cascadia Code', 'Fira Code', 'JetBrains Mono', 'Consolas', monospace;
            font-size: 14px;
            line-height: 1.4;
            padding: 16px;
            overflow-x: hidden;
        }
        .tui-container {
            white-space: pre;
            tab-size: 4;
        }
        .cursor {
            background-color: ${theme.cursor};
            color: ${theme.background};
            animation: blink 1s step-end infinite;
        }
        @keyframes blink {
            50% { opacity: 0; }
        }
        .ansi-bold { font-weight: bold; }
        .ansi-italic { font-style: italic; }
        .ansi-underline { text-decoration: underline; }
        .ansi-fg-black { color: #000000; }
        .ansi-fg-red { color: #CC0000; }
        .ansi-fg-green { color: #4E9A06; }
        .ansi-fg-yellow { color: #C4A000; }
        .ansi-fg-blue { color: #3465A4; }
        .ansi-fg-magenta { color: #75507B; }
        .ansi-fg-cyan { color: #06989A; }
        .ansi-fg-white { color: #D3D7CF; }
        .ansi-bg-black { background-color: #000000; }
        .ansi-bg-red { background-color: #CC0000; }
        .ansi-bg-green { background-color: #4E9A06; }
        .ansi-bg-yellow { background-color: #C4A000; }
        .ansi-bg-blue { background-color: #3465A4; }
        .ansi-bg-magenta { background-color: #75507B; }
        .ansi-bg-cyan { background-color: #06989A; }
        .ansi-bg-white { background-color: #D3D7CF; }
        .scroll-container {
            width: 100%;
            height: calc(100vh - 32px);
            overflow: auto;
        }
    </style>
</head>
<body>
    <div class="scroll-container">
        <div class="tui-container" id="output">${styledOutput}</div>
    </div>
    <script>
        // Auto-scroll to bottom
        const container = document.querySelector('.scroll-container');
        if (container) {
            container.scrollTop = container.scrollHeight;
        }
    </script>
</body>
</html>`;
    }
    convertToStyledHtml(output, _theme) {
        // Escape HTML entities
        let html = output
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;');
        // Basic ANSI color mapping
        const ansiPatterns = [
            [/\[1m/g, '<span class="ansi-bold">'],
            [/\[3m/g, '<span class="ansi-italic">'],
            [/\[4m/g, '<span class="ansi-underline">'],
            [/\[0m/g, '</span>'],
            [/\[30m/g, '<span class="ansi-fg-black">'],
            [/\[31m/g, '<span class="ansi-fg-red">'],
            [/\[32m/g, '<span class="ansi-fg-green">'],
            [/\[33m/g, '<span class="ansi-fg-yellow">'],
            [/\[34m/g, '<span class="ansi-fg-blue">'],
            [/\[35m/g, '<span class="ansi-fg-magenta">'],
            [/\[36m/g, '<span class="ansi-fg-cyan">'],
            [/\[37m/g, '<span class="ansi-fg-white">'],
            [/\[90m/g, '<span class="ansi-fg-black">'],
            [/\[91m/g, '<span class="ansi-fg-red">'],
            [/\[92m/g, '<span class="ansi-fg-green">'],
            [/\[93m/g, '<span class="ansi-fg-yellow">'],
            [/\[94m/g, '<span class="ansi-fg-blue">'],
            [/\[95m/g, '<span class="ansi-fg-magenta">'],
            [/\[96m/g, '<span class="ansi-fg-cyan">'],
            [/\[97m/g, '<span class="ansi-fg-white">'],
        ];
        for (const [pattern, replacement] of ansiPatterns) {
            html = html.replace(pattern, replacement);
        }
        // Convert newlines to <br>
        html = html.replace(/\n/g, '<br>');
        // Add cursor styling
        html = html.replace(/█/g, '<span class="cursor">█</span>');
        return html;
    }
    get onDidChange() {
        return this._changeEvent.event;
    }
    dispose() {
        this._changeEvent.dispose();
        this._emitter.removeAllListeners();
    }
}
// ============================================================================
// LSP Client
// ============================================================================
class DraconLspClient {
    constructor(workspaceRoot) {
        this.process = null;
        this.onOutputCallback = null;
        this.onErrorCallback = null;
        this.onExitCallback = null;
        this.workspaceRoot = workspaceRoot;
    }
    start(examplePath, onOutput, onError, onExit) {
        this.onOutputCallback = onOutput;
        this.onErrorCallback = onError;
        this.onExitCallback = onExit;
        const config = vscode.workspace.getConfiguration('dracon');
        const lspServerPath = this.resolvePath(config.get('lspServerPath', '') || 'target/debug/dracon-lsp');
        // Build LSP server arguments
        const args = [
            '--stdio',
            '--workspace-root',
            this.workspaceRoot,
            '--example',
            examplePath
        ];
        try {
            this.process = cp.spawn(lspServerPath, args, {
                cwd: this.workspaceRoot,
                env: {
                    ...process.env,
                    'RUST_BACKTRACE': '1'
                }
            });
            this.process.stdout?.on('data', (data) => {
                if (this.onOutputCallback) {
                    this.onOutputCallback(data.toString());
                }
            });
            this.process.stderr?.on('data', (data) => {
                if (this.onErrorCallback) {
                    this.onErrorCallback(data.toString());
                }
            });
            this.process.on('exit', (code) => {
                if (this.onExitCallback) {
                    this.onExitCallback(code ?? -1);
                }
            });
            this.process.on('error', (err) => {
                if (this.onErrorCallback) {
                    this.onErrorCallback(`Failed to start LSP server: ${err.message}`);
                }
            });
            // Send initialize request
            this.sendRequest('initialize', {
                processId: process.pid,
                rootUri: vscode.Uri.file(this.workspaceRoot).toString(),
                capabilities: {}
            });
        }
        catch (err) {
            if (this.onErrorCallback) {
                this.onErrorCallback(`Failed to spawn LSP server: ${err}`);
            }
        }
    }
    sendRequest(method, params) {
        if (this.process && this.process.stdin) {
            const request = JSON.stringify({
                jsonrpc: '2.0',
                id: Date.now(),
                method,
                params
            });
            this.process.stdin.write(`Content-Length: ${request.length}\r\n\r\n${request}`);
        }
    }
    stop() {
        if (this.process) {
            this.process.kill('SIGTERM');
            this.process = null;
        }
    }
    sendNotification(method, params) {
        if (this.process && this.process.stdin) {
            const notification = JSON.stringify({
                jsonrpc: '2.0',
                method,
                params
            });
            this.process.stdin.write(`Content-Length: ${notification.length}\r\n\r\n${notification}`);
        }
    }
    resolvePath(p) {
        if (path.isAbsolute(p)) {
            return p;
        }
        return path.join(this.workspaceRoot, p);
    }
}
// ============================================================================
// Preview Manager
// ============================================================================
class PreviewManager {
    constructor(context) {
        this.previewPanel = null;
        this.lspClient = null;
        this.currentExamplePath = '';
        this.decorationCollection = [];
        this.terminalOutput = '';
        this.terminalBuffer = '';
        this.animationFrame = null;
        this.contentProvider = new DraconTUIContentProvider();
        // Create status bar item
        this.statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
        this.statusBarItem.text = '$(play) Dracon Preview';
        this.statusBarItem.command = 'dracon.startPreview';
        this.statusBarItem.tooltip = 'Start TUI Preview';
        this.statusBarItem.show();
        // Register content provider
        context.subscriptions.push(vscode.workspace.registerTextDocumentContentProvider('dracon-tui', this.contentProvider));
        // Register command handlers
        context.subscriptions.push(vscode.commands.registerCommand('dracon.startPreview', () => this.startPreview()), vscode.commands.registerCommand('dracon.stopPreview', () => this.stopPreview()), vscode.commands.registerCommand('dracon.refreshPreview', () => this.refreshPreview()));
        // Listen for document changes to auto-refresh
        context.subscriptions.push(vscode.workspace.onDidChangeTextDocument((event) => {
            if (this.lspClient && this.isRustFile(event.document)) {
                this.onDocumentChange(event.document);
            }
        }));
        // Listen for terminal close
        context.subscriptions.push(vscode.window.onDidCloseTerminal((terminal) => {
            if (terminal.name === 'Dracon Preview') {
                this.stopPreview();
            }
        }));
    }
    isRustFile(document) {
        return document.languageId === 'rust' &&
            document.uri.scheme === 'file' &&
            document.fileName.endsWith('.rs');
    }
    async startPreview() {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showWarningMessage('No active editor');
            return;
        }
        const document = editor.document;
        if (!this.isRustFile(document)) {
            vscode.window.showWarningMessage('Please open a Rust file');
            return;
        }
        const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
        if (!workspaceRoot) {
            vscode.window.showWarningMessage('No workspace folder found');
            return;
        }
        // Stop any existing preview
        this.stopPreview();
        this.currentExamplePath = document.uri.fsPath;
        // Update status
        this.statusBarItem.text = '$(sync~spin) Building...';
        this.statusBarItem.command = 'dracon.stopPreview';
        // Show preview panel
        this.previewPanel = vscode.window.createWebviewPanel('draconTuiPreview', 'Dracon TUI Preview', vscode.ViewColumn.Two, {
            enableScripts: true,
            retainContextWhenHidden: true,
            localResourceRoots: []
        });
        // Handle panel close
        this.previewPanel.onDidDispose(() => {
            this.stopPreview();
        });
        // Create LSP client
        this.lspClient = new DraconLspClient(workspaceRoot);
        this.lspClient.start(this.currentExamplePath, (output) => this.onLspOutput(output), (error) => this.onLspError(error), (code) => this.onLspExit(code));
        // Send didOpen notification after initialization
        setTimeout(() => {
            if (this.lspClient) {
                this.lspClient.sendNotification('textDocument/didOpen', {
                    textDocument: {
                        uri: document.uri.toString(),
                        languageId: 'rust',
                        version: 1,
                        text: document.getText()
                    }
                });
            }
        }, 500);
        // Start render loop
        this.startRenderLoop();
    }
    onLspOutput(output) {
        this.terminalBuffer += output;
        this.processBuffer();
    }
    onLspError(error) {
        console.error('LSP Error:', error);
        this.appendToTerminal(`\r\n\x1b[31mError: ${error}\x1b[0m\r\n`);
    }
    onLspExit(code) {
        this.appendToTerminal(`\r\n\x1b[33mProcess exited with code ${code}\x1b[0m\r\n`);
        this.statusBarItem.text = `$(error) Exit: ${code}`;
        this.statusBarItem.command = 'dracon.refreshPreview';
    }
    processBuffer() {
        // Process complete lines from buffer
        const lines = this.terminalBuffer.split('\n');
        this.terminalBuffer = lines.pop() || '';
        for (const line of lines) {
            this.appendToTerminal(line + '\n');
        }
    }
    appendToTerminal(text) {
        this.terminalOutput += text;
        // Update preview if panel exists
        if (this.previewPanel) {
            const theme = vscode.workspace.getConfiguration('dracon').get('theme', 'nord');
            const themeConfig = THEME_COLORS[theme] || THEME_COLORS.nord;
            this.previewPanel.webview.html = this.generatePreviewHtml(themeConfig);
        }
    }
    startRenderLoop() {
        if (this.animationFrame) {
            clearInterval(this.animationFrame);
        }
        this.animationFrame = setInterval(() => {
            if (this.terminalBuffer.length > 0) {
                this.processBuffer();
            }
        }, 100); // Process at 10fps
        this.statusBarItem.text = '$(pass) Preview Active';
    }
    stopPreview() {
        if (this.animationFrame) {
            clearInterval(this.animationFrame);
            this.animationFrame = null;
        }
        if (this.lspClient) {
            this.lspClient.stop();
            this.lspClient = null;
        }
        if (this.previewPanel) {
            this.previewPanel.dispose();
            this.previewPanel = null;
        }
        this.terminalOutput = '';
        this.terminalBuffer = '';
        this.statusBarItem.text = '$(play) Dracon Preview';
        this.statusBarItem.command = 'dracon.startPreview';
    }
    refreshPreview() {
        this.stopPreview();
        this.startPreview();
    }
    onDocumentChange(document) {
        if (this.lspClient) {
            // Debounce document changes
            clearTimeout(this._debounceTimer);
            this._debounceTimer = setTimeout(() => {
                if (this.lspClient) {
                    this.lspClient.sendNotification('textDocument/didChange', {
                        textDocument: {
                            uri: document.uri.toString(),
                            version: document.version
                        },
                        contentChanges: [{
                                text: document.getText()
                            }]
                    });
                }
            }, 300);
        }
    }
    generatePreviewHtml(theme) {
        const styledOutput = this.convertToStyledOutput(this.terminalOutput, theme);
        return `<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        body {
            background-color: ${theme.background};
            color: ${theme.foreground};
            font-family: 'Cascadia Code', 'Fira Code', 'JetBrains Mono', 'Consolas', monospace;
            font-size: 14px;
            line-height: 1.4;
            padding: 16px;
            overflow-x: hidden;
            min-height: 100vh;
        }
        .tui-container {
            white-space: pre;
            tab-size: 4;
            word-wrap: normal;
        }
        .cursor {
            background-color: ${theme.cursor};
            color: ${theme.background};
            animation: blink 1s step-end infinite;
        }
        @keyframes blink {
            50% { opacity: 0; }
        }
        .ansi-bold { font-weight: bold; }
        .ansi-italic { font-style: italic; }
        .ansi-underline { text-decoration: underline; }
        .ansi-fg-black { color: #000000; }
        .ansi-fg-red { color: #CC0000; }
        .ansi-fg-green { color: #4E9A06; }
        .ansi-fg-yellow { color: #C4A000; }
        .ansi-fg-blue { color: #3465A4; }
        .ansi-fg-magenta { color: #75507B; }
        .ansi-fg-cyan { color: #06989A; }
        .ansi-fg-white { color: #D3D7CF; }
        .ansi-fg-bright-black { color: #555555; }
        .ansi-fg-bright-red { color: #EF2929; }
        .ansi-fg-bright-green { color: #8AE234; }
        .ansi-fg-bright-yellow { color: #FCE94F; }
        .ansi-fg-bright-blue { color: #729FCF; }
        .ansi-fg-bright-magenta { color: #AD7FA8; }
        .ansi-fg-bright-cyan { color: #34E2E2; }
        .ansi-fg-bright-white { color: #FFFFFF; }
        .ansi-bg-black { background-color: #000000; }
        .ansi-bg-red { background-color: #CC0000; }
        .ansi-bg-green { background-color: #4E9A06; }
        .ansi-bg-yellow { background-color: #C4A000; }
        .ansi-bg-blue { background-color: #3465A4; }
        .ansi-bg-magenta { background-color: #75507B; }
        .ansi-bg-cyan { background-color: #06989A; }
        .ansi-bg-white { background-color: #D3D7CF; }
        .scroll-container {
            width: 100%;
            height: 100vh;
            overflow: auto;
        }
    </style>
</head>
<body>
    <div class="scroll-container">
        <div class="tui-container" id="output">${styledOutput}</div>
    </div>
    <script>
        // Auto-scroll to bottom on new content
        const container = document.querySelector('.scroll-container');
        if (container) {
            const observer = new MutationObserver(() => {
                container.scrollTop = container.scrollHeight;
            });
            observer.observe(document.getElementById('output'), { childList: true, subtree: true });
        }
    </script>
</body>
</html>`;
    }
    convertToStyledOutput(output, _theme) {
        // Escape HTML entities
        let html = output
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;');
        // ANSI escape sequence patterns
        const ansiPatterns = [
            [/\[1m/g, '<span class="ansi-bold">'],
            [/\[3m/g, '<span class="ansi-italic">'],
            [/\[4m/g, '<span class="ansi-underline">'],
            [/\[0m/g, '</span>'],
            [/\[30m/g, '<span class="ansi-fg-black">'],
            [/\[31m/g, '<span class="ansi-fg-red">'],
            [/\[32m/g, '<span class="ansi-fg-green">'],
            [/\[33m/g, '<span class="ansi-fg-yellow">'],
            [/\[34m/g, '<span class="ansi-fg-blue">'],
            [/\[35m/g, '<span class="ansi-fg-magenta">'],
            [/\[36m/g, '<span class="ansi-fg-cyan">'],
            [/\[37m/g, '<span class="ansi-fg-white">'],
            [/\[90m/g, '<span class="ansi-fg-bright-black">'],
            [/\[91m/g, '<span class="ansi-fg-bright-red">'],
            [/\[92m/g, '<span class="ansi-fg-bright-green">'],
            [/\[93m/g, '<span class="ansi-fg-bright-yellow">'],
            [/\[94m/g, '<span class="ansi-fg-bright-blue">'],
            [/\[95m/g, '<span class="ansi-fg-bright-magenta">'],
            [/\[96m/g, '<span class="ansi-fg-bright-cyan">'],
            [/\[97m/g, '<span class="ansi-fg-bright-white">'],
            // Close all spans on newlines for performance
            [/\n/g, '</span>\n<span>'],
        ];
        for (const [pattern, replacement] of ansiPatterns) {
            html = html.replace(pattern, replacement);
        }
        // Wrap in span for styling
        html = '<span>' + html + '</span>';
        // Add cursor styling
        html = html.replace(/█/g, '<span class="cursor">█</span>');
        // Clean up empty spans
        html = html.replace(/<span><\/span>/g, '');
        return html;
    }
}
// ============================================================================
// Extension Entry Point
// ============================================================================
let previewManager;
function activate(context) {
    console.log('Dracon TUI Preview extension activated');
    // Initialize preview manager
    previewManager = new PreviewManager(context);
    // Register status bar commands
    context.subscriptions.push(vscode.commands.registerCommand('dracon.showcase', async () => {
        const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
        if (!workspaceRoot) {
            vscode.window.showWarningMessage('No workspace folder found');
            return;
        }
        const config = vscode.workspace.getConfiguration('dracon');
        const showcasePath = config.get('showcasePath', 'target/debug/showcase');
        const theme = config.get('theme', 'nord');
        // Resolve showcase path
        const resolvedPath = path.isAbsolute(showcasePath)
            ? showcasePath
            : path.join(workspaceRoot, showcasePath);
        // Spawn showcase process
        const showcaseProcess = cp.spawn(resolvedPath, [], {
            cwd: workspaceRoot,
            env: {
                ...process.env,
                'DTRON_THEME': theme
            },
            shell: true
        });
        // Create terminal for output
        const terminal = vscode.window.createTerminal({
            name: 'Dracon Preview',
            cwd: workspaceRoot
        });
        showcaseProcess.stdout?.on('data', (data) => {
            terminal.sendText(data.toString(), false);
        });
        showcaseProcess.stderr?.on('data', (data) => {
            terminal.sendText(data.toString(), false);
        });
        showcaseProcess.on('exit', (code) => {
            terminal.sendText(`\r\n\x1b[33m[Process exited with code ${code}]\x1b[0m\r\n`, false);
        });
        terminal.show();
    }));
    // Register completion provider for Rust files
    context.subscriptions.push(vscode.languages.registerCompletionItemProvider('rust', {
        provideCompletionItems: (document, position) => {
            const line = document.lineAt(position.line).text;
            const word = line.substring(0, position.character);
            // Provide Dracon-specific completions
            const completions = [];
            // Example completions
            const examples = [
                { label: 'dracon::examples::showcase', detail: 'Run the showcase example' },
                { label: 'dracon::examples::widget_gallery', detail: 'Show widget gallery' },
                { label: 'dracon::examples::form_demo', detail: 'Form demo example' },
            ];
            for (const example of examples) {
                if (word.includes('dracon') || word.includes('Example')) {
                    const item = new vscode.CompletionItem(example.label, vscode.CompletionItemKind.Module);
                    item.detail = example.detail;
                    completions.push(item);
                }
            }
            return completions;
        }
    }));
    // Register code lens for run commands
    context.subscriptions.push(vscode.languages.registerCodeLensProvider('rust', {
        provideCodeLenses: (document) => {
            const lenses = [];
            // Add code lens for main functions
            const text = document.getText();
            const regex = /#\[test\]|fn main\(\)/g;
            let match;
            while ((match = regex.exec(text)) !== null) {
                const position = document.positionAt(match.index);
                const range = new vscode.Range(position, position);
                const lens = new vscode.CodeLens(range, {
                    title: '$(play) Run Preview',
                    command: 'dracon.startPreview',
                    arguments: [document.uri]
                });
                lenses.push(lens);
            }
            return lenses;
        }
    }));
}
function deactivate() {
    if (previewManager) {
        previewManager.stop();
    }
}
//# sourceMappingURL=extension.js.map
import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { LanguageClient, LanguageClientOptions, ServerOptions, TransportKind } from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
    const serverModule = context.asAbsolutePath(path.join('rezero-studio', 'out', 'server.js'));
    const serverOptions: ServerOptions = {
        run: { module: serverModule, transport: TransportKind.ipc },
        debug: { module: serverModule, transport: TransportKind.ipc }
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'rezero' }],
    };

    client = new LanguageClient('rezeroLanguageServer', 'ReZero Language Server', serverOptions, clientOptions);
    client.start();

    const provider = new EntropyProfilerProvider(context.extensionUri);
    context.subscriptions.push(vscode.window.registerWebviewViewProvider('entropyProfiler', provider));

    client.onReady().then(() => {
        client.onNotification("rezero/energyProfile", (data) => {
            provider.updateEntropyData(data);
        });
    });

    let debugState = { pc: 0, lrsStack: [] as any[], history: [] as any[] };

    context.subscriptions.push(
        vscode.commands.registerCommand('rezero.debugStepForward', () => {
            debugState.history.push(JSON.parse(JSON.stringify(debugState)));
            debugState.pc += 1;
            debugState.lrsStack.push({ name: `Var_${debugState.pc}`, val: Math.floor(Math.random() * 100) });
            provider.postMessageToWebview({ type: 'updateDebug', pc: debugState.pc, stack: debugState.lrsStack });
        }),
        vscode.commands.registerCommand('rezero.debugStepBackward', () => {
            if (debugState.history.length > 0) {
                debugState = debugState.history.pop();
                provider.postMessageToWebview({ type: 'updateDebug', pc: debugState.pc, stack: debugState.lrsStack });
            }
        })
    );
}

class EntropyProfilerProvider implements vscode.WebviewViewProvider {
    private _view?: vscode.WebviewView;
    constructor(private readonly _extensionUri: vscode.Uri) {}

    resolveWebviewView(webviewView: vscode.WebviewView) {
        this._view = webviewView;
        webviewView.webview.options = { enableScripts: true };
        const htmlPath = path.join(this._extensionUri.fsPath, 'rezero-studio', 'webview', 'entropy-profiler.html');
        webviewView.webview.html = fs.readFileSync(htmlPath, 'utf8');
    }

    public updateEntropyData(data: any) { this._view?.webview.postMessage({ type: 'entropyUpdate', data }); }
    public postMessageToWebview(message: any) { this._view?.webview.postMessage(message); }
}

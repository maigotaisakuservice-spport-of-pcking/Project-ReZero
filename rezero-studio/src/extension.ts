import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { LanguageClient, TransportKind } from 'vscode-languageclient/node';

export function activate(context: vscode.ExtensionContext) {
    const serverModule = context.asAbsolutePath(path.join('rezero-studio', 'out', 'server.js'));
    const client = new LanguageClient('rezeroLS', 'ReZero LS', {
        run: { module: serverModule, transport: TransportKind.ipc },
        debug: { module: serverModule, transport: TransportKind.ipc }
    }, {
        documentSelector: [{ scheme: 'file', language: 'rezero' }]
    });
    client.start();

    const provider = new EntropyProfilerProvider(context.extensionUri);
    context.subscriptions.push(vscode.window.registerWebviewViewProvider('entropyProfiler', provider));

    client.onReady().then(() => {
        client.onNotification("rezero/energyProfile", (data) => {
            provider.update(data);
        });
    });

    // Simple Debugger Logic
    context.subscriptions.push(vscode.commands.registerCommand('rezero.stepForward', () => {
        provider.post({ type: 'updateDebug', pc: 1, stack: [{name: 'temp', val: 12}] });
    }));
}

class EntropyProfilerProvider implements vscode.WebviewViewProvider {
    private _view?: vscode.WebviewView;
    constructor(private readonly _uri: vscode.Uri) {}
    resolveWebviewView(view: vscode.WebviewView) {
        this._view = view;
        view.webview.options = { enableScripts: true };
        const html = fs.readFileSync(path.join(this._uri.fsPath, 'rezero-studio', 'webview', 'entropy-profiler.html'), 'utf8');
        view.webview.html = html;
    }
    update(data: any) { this._view?.webview.postMessage({ type: 'entropyUpdate', data }); }
    post(msg: any) { this._view?.webview.postMessage(msg); }
}

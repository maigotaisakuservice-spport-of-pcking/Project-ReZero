import { createConnection, TextDocuments, Diagnostic, ProposedFeatures, TextDocumentSyncKind } from 'vscode-languageserver/node';
import { TextDocument } from 'vscode-languageserver-textdocument';
import { exec } from 'child_process';

const connection = createConnection(ProposedFeatures.all);
const documents: TextDocuments<TextDocument> = new TextDocuments(TextDocument);

connection.onInitialize(() => ({ capabilities: { textDocumentSync: TextDocumentSyncKind.Incremental } }));

documents.onDidChangeContent(change => {
    const text = change.document.getText();
    const cp = exec(`rezero-core verify dummy.rz --stdin`, (err, stdout) => {
        const diagnostics: Diagnostic[] = [];
        try {
            const res = JSON.parse(stdout);
            if (res.status === "verified_failed") {
                diagnostics.push({
                    range: { start: { line: res.error.line, character: 0 }, end: { line: res.error.line, character: 100 } },
                    message: res.error.message,
                    source: 'ReZero'
                });
            }
        } catch(e) {}
        connection.sendDiagnostics({ uri: change.document.uri, diagnostics });
    });
    cp.stdin?.write(text);
    cp.stdin?.end();
});

documents.listen(connection);
connection.listen();

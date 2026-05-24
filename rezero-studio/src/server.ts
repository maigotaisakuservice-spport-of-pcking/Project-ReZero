import {
    createConnection,
    TextDocuments,
    Diagnostic,
    DiagnosticSeverity,
    ProposedFeatures,
    InitializeParams,
    TextDocumentSyncKind,
    TextDocumentChangeEvent
} from 'vscode-languageserver/node';
import { TextDocument } from 'vscode-languageserver-textdocument';
import { exec } from 'child_process';

const connection = createConnection(ProposedFeatures.all);
const documents: TextDocuments<TextDocument> = new TextDocuments(TextDocument);

connection.onInitialize((params: InitializeParams) => {
    return {
        capabilities: {
            textDocumentSync: TextDocumentSyncKind.Incremental,
        }
    };
});

documents.onDidChangeContent((change: TextDocumentChangeEvent<TextDocument>) => {
    validateTextDocument(change.document);
});

async function validateTextDocument(textDocument: TextDocument): Promise<void> {
    const text = textDocument.getText();
    const diagnostics: Diagnostic[] = [];

    const cp = exec(`rezero-core verify dummy.rz --stdin`, (err, stdout, stderr) => {
        try {
            const result = JSON.parse(stdout);
            if (result.status === "verified_failed") {
                const diagnostic: Diagnostic = {
                    severity: DiagnosticSeverity.Error,
                    range: {
                        start: { line: result.error.line, character: result.error.character },
                        end: { line: result.error.line, character: 100 }
                    },
                    message: `Z3 Proof Failed: ${result.error.message}`,
                    source: 'ReZero SMT'
                };
                diagnostics.push(diagnostic);
            }
        } catch (e) {}

        const lines = text.split('\n');
        const assignmentsCount = lines.filter(line => line.includes('=') && !line.includes('==')).length;
        const estimatedEntropy = assignmentsCount * 1.38e-23 * Math.log(2);

        connection.sendDiagnostics({ uri: textDocument.uri, diagnostics });
        connection.sendNotification("rezero/energyProfile", {
            uri: textDocument.uri,
            assignmentsCount,
            entropy: estimatedEntropy
        });
    });

    cp.stdin?.write(text);
    cp.stdin?.end();
}

documents.listen(connection);
connection.listen();

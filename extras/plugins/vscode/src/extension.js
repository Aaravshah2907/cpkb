const vscode = require('vscode');
const cp = require('child_process');

function activate(context) {
    let disposable = vscode.commands.registerCommand('cpkb.insertSnippet', async () => {
        
        // 1. Query CPKB for all snippets
        cp.exec('cpkb query "" --limit 1000', (err, stdout, stderr) => {
            if (err) {
                vscode.window.showErrorMessage('Failed to query cpkb. Is it installed and in your PATH?');
                return;
            }

            const lines = stdout.trim().split('\n').filter(l => l.length > 0);
            const quickPickItems = lines.map(line => {
                const parts = line.split('|');
                return {
                    label: parts[0].trim(),
                    description: parts.slice(1).join('|').trim()
                };
            });

            if (quickPickItems.length === 0) {
                vscode.window.showInformationMessage('No CPKB snippets found.');
                return;
            }

            // 2. Show QuickPick
            vscode.window.showQuickPick(quickPickItems, {
                placeHolder: 'Search CPKB Snippets',
                matchOnDescription: true
            }).then(selection => {
                if (!selection) return;

                // 3. Fetch the selected snippet's content
                cp.exec(`cpkb show ${selection.label}`, (errShow, stdoutShow, stderrShow) => {
                    if (errShow) {
                        vscode.window.showErrorMessage(`Failed to fetch snippet ${selection.label}`);
                        return;
                    }

                    // Extract the code block (everything after "--- Code ---")
                    const codeMatch = stdoutShow.match(/---\s*Code\s*---([\s\S]*?)------------/);
                    const code = codeMatch ? codeMatch[1].trim() : stdoutShow.trim();

                    // 4. Insert at active text editor
                    const editor = vscode.window.activeTextEditor;
                    if (editor) {
                        editor.edit(editBuilder => {
                            editBuilder.insert(editor.selection.active, code);
                        });
                        vscode.window.showInformationMessage(`Inserted snippet: ${selection.label}`);
                    }
                });
            });
        });
    });

    context.subscriptions.push(disposable);
}

function deactivate() {}

module.exports = {
    activate,
    deactivate
};

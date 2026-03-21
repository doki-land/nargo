import * as vscode from 'vscode';
import * as path from 'path';
import { LanguageClient, LanguageClientOptions, ServerOptions, TransportKind } from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
    // 启动语言服务器
    const serverModule = context.asAbsolutePath(path.join('..', '..', 'target', 'debug', 'nargo-lsp.exe'));
    const debugOptions = {
        execArgv: ['--nolazy', '--inspect=6009']
    };
    
    const serverOptions: ServerOptions = {
        run: { module: serverModule, transport: TransportKind.stdio },
        debug: { module: serverModule, transport: TransportKind.stdio, options: debugOptions }
    };
    
    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'nargo' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.{nargo,oak}')
        }
    };
    
    client = new LanguageClient(
        'nargo-lsp',
        'Nargo Language Server',
        serverOptions,
        clientOptions
    );
    
    client.start();
    
    // 注册命令
    context.subscriptions.push(
        vscode.commands.registerCommand('nargo.initialize', async () => {
            vscode.window.showInformationMessage('Nargo initialized!');
        }),
        
        vscode.commands.registerCommand('nargo.build', async () => {
            const terminal = vscode.window.createTerminal('Nargo Build');
            terminal.sendText('nargo build');
            terminal.show();
        }),
        
        vscode.commands.registerCommand('nargo.run', async () => {
            const terminal = vscode.window.createTerminal('Nargo Run');
            terminal.sendText('nargo run');
            terminal.show();
        }),
        
        vscode.commands.registerCommand('nargo.test', async () => {
            const terminal = vscode.window.createTerminal('Nargo Test');
            terminal.sendText('nargo test');
            terminal.show();
        })
    );
}

export function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
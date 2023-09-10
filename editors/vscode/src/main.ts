/* --------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See License.txt in the project root for license information.
 * ------------------------------------------------------------------------------------------ */

import * as path from "path";
import {workspace, ExtensionContext, window, DocumentFilter} from 'vscode';

import {
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
	TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
  const outputChannel = window.createOutputChannel("Ruff");
  const traceOutputChannel = window.createOutputChannel("Ruff Trace");

  outputChannel.appendLine("Starting Ruff LSP Client");

	// If the extension is launched in debug mode then the debug server options are used
	// Otherwise the run options are used
	const serverOptions: ServerOptions = {command: "/Users/micha/astral/ruff/target/debug/ruff", args: ["lsp"], };

  const documentSelector: DocumentFilter[] = [
    { language: "python", scheme: "file" },
  ];

	// Options to control the language client
	const clientOptions: LanguageClientOptions = {
		documentSelector: documentSelector,
    outputChannel,
    traceOutputChannel
	};

	// Create the language client and start the client.
	client = new LanguageClient(
		'ruff',
		'Ruff',
		serverOptions,
		clientOptions
	);

	// Start the client. This will also launch the server
	client.start().catch((error) => {

    outputChannel.show();
    outputChannel.appendLine(`Failed to start LSP Client: ${error}`);
  })
}

export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined;
	}
	return client.stop();
}

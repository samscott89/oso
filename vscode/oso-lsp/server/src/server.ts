/* --------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See License.txt in the project root for license information.
 * ------------------------------------------------------------------------------------------ */
import {
	createConnection,
	TextDocuments,
	Diagnostic,
	DiagnosticSeverity,
	ProposedFeatures,
	InitializeParams,
	DidChangeConfigurationNotification,
	CompletionItem,
	CompletionItemKind,
	TextDocumentPositionParams,
	TextDocumentSyncKind,
	InitializeResult,
	integer,
	DocumentSymbolParams,
	SymbolKind,
	SymbolInformation,
	Position,
	Range

} from 'vscode-languageserver/node';

import {
	DocumentUri,
	TextDocument
} from 'vscode-languageserver-textdocument';
import { Polar } from './polar_analyzer';
import { create } from 'domain';

// Create a connection for the server, using Node's IPC as a transport.
// Also include all preview / proposed LSP features.
const connection = createConnection(ProposedFeatures.all);

const polar = new Polar();

// Create a simple text document manager.
const documents: TextDocuments<TextDocument> = new TextDocuments(TextDocument);

let hasConfigurationCapability = false;
let hasWorkspaceFolderCapability = false;
let hasDiagnosticRelatedInformationCapability = false;

connection.onInitialize((params: InitializeParams) => {
	const capabilities = params.capabilities;

	// Does the client support the `workspace/configuration` request?
	// If not, we fall back using global settings.
	hasConfigurationCapability = !!(
		capabilities.workspace && !!capabilities.workspace.configuration
	);
	hasWorkspaceFolderCapability = !!(
		capabilities.workspace && !!capabilities.workspace.workspaceFolders
	);
	hasDiagnosticRelatedInformationCapability = !!(
		capabilities.textDocument &&
		capabilities.textDocument.publishDiagnostics &&
		capabilities.textDocument.publishDiagnostics.relatedInformation
	);

	const result: InitializeResult = {
		capabilities: {
			textDocumentSync: TextDocumentSyncKind.Incremental,
			// Tell the client that this server supports code completion.
			completionProvider: {
				resolveProvider: true
			},
			documentSymbolProvider: true
		}
	};
	if (hasWorkspaceFolderCapability) {
		result.capabilities.workspace = {
			workspaceFolders: {
				supported: true
			}
		};
	}
	return result;
});

connection.onInitialized(() => {
	if (hasConfigurationCapability) {
		// Register for all configuration changes.
		connection.client.register(DidChangeConfigurationNotification.type, undefined);
	}
	if (hasWorkspaceFolderCapability) {
		connection.workspace.onDidChangeWorkspaceFolders(_event => {
			connection.console.log('Workspace folder change event received.');
		});
	}
});

// The example settings
interface Settings {
	maxNumberOfProblems: number;
}

// The global settings, used when the `workspace/configuration` request is not supported by the client.
// Please note that this is not the case when using this server with the client provided in this example
// but could happen with other clients.
const defaultSettings: Settings = { maxNumberOfProblems: 1000 };
let globalSettings: Settings = defaultSettings;

// Cache the settings of all open documents
const documentSettings: Map<string, Thenable<Settings>> = new Map();

connection.onDidChangeConfiguration(change => {
	if (hasConfigurationCapability) {
		// Reset all cached document settings
		documentSettings.clear();
	} else {
		globalSettings = <Settings>(
			(change.settings.osoLsp || defaultSettings)
		);
	}

	// Revalidate all open text documents
	documents.all().forEach(validateContents);
});

function getDocumentSettings(resource: string): Thenable<Settings> {
	if (!hasConfigurationCapability) {
		return Promise.resolve(globalSettings);
	}
	let result = documentSettings.get(resource);
	if (!result) {
		result = connection.workspace.getConfiguration({
			scopeUri: resource,
			section: 'osoLsp'
		});
		documentSettings.set(resource, result);
	}
	return result;
}

// Only keep settings for open documents
documents.onDidClose(e => {
	documentSettings.delete(e.document.uri);
});

// Attempt to load the policy file into the Polar knowledge base
// Any errors will be returned to the client.
//
// returns `false` if loading fails. In which case the knowledge
// base will be left unchanged
function tryLoadFile(textDocument: TextDocument): boolean {
	const policy = textDocument.getText();
	const errors = polar.getParseErrors(policy);
	const diagnostics: Diagnostic[] = [];
	var success = false;
	if (errors.length === 0) {
		// no parse errors! Lets try loading the policy for real
		const filename = textDocument.uri;
		try {
			polar.load(policy, filename);
			success = true;
		} catch (error) {
			const diagnostic: Diagnostic = {
				severity: DiagnosticSeverity.Error,
				message: error,
				range: {
					start: textDocument.positionAt(-1),
					end: textDocument.positionAt(-1)
				},
				source: 'polar'
			};
			diagnostics.push(diagnostic);
		}
	} else {
		// parse errors :( 
		// send them back to the client
		errors.forEach(([message, left, right]: [string, integer, integer]) => {
			const diagnostic: Diagnostic = {
				severity: DiagnosticSeverity.Error,
				message: message,
				range: {
					start: textDocument.positionAt(left),
					end: textDocument.positionAt(right)
				},
				source: 'polar'
			};
			diagnostics.push(diagnostic);
		});

	}
	connection.sendDiagnostics({ uri: textDocument.uri, diagnostics });
	return success
}

// The content of a text document has changed. This event is emitted
// when the text document first opened or when its content has changed.
documents.onDidChangeContent(change => {
	console.log("Document change: ", change)
	const doc = change.document
	if (tryLoadFile(doc)) {
		validateContents(doc);
	}
});

async function validateContents(textDocument: TextDocument): Promise<void> {
	const diagnostics: Diagnostic[] = [];
	const policy = textDocument.getText()
	const unused_rules = polar.getUnusedRules(policy);
	unused_rules.forEach(([ruleName, left, right]: [string, integer, integer]) => {
		const diagnostic: Diagnostic = {
			severity: DiagnosticSeverity.Warning,
			message: `Rule does not exist: ${ruleName}`,
			range: {
				start: textDocument.positionAt(left),
				end: textDocument.positionAt(right)
			},
			source: 'polar'
		};
		diagnostics.push(diagnostic);
	});
	connection.sendDiagnostics({ uri: textDocument.uri, diagnostics });
}

connection.onDocumentSymbol((params: DocumentSymbolParams): SymbolInformation[] => {
	connection.console.log('We received a document symbol event');
	const doc = documents.get(params.textDocument.uri);
	console.log(`doc is ${doc}`);
	const result: SymbolInformation[] = [];

	if (doc !== undefined) {
		const rules: {
			symbol: string,
			signature: string,
			location: [string, number, number]
		}[]
			= polar.getRuleInfo();

		console.log('Polar rules found are', rules);
		rules.forEach((rule: {
			symbol: string,
			signature: string,
			location: [string, number, number]
		}) => {

			const currentDocUri: DocumentUri = rule.location[0];

			if (currentDocUri === params.textDocument.uri) {
				const symbolSummary: SymbolInformation = {
					name: rule.symbol,
					kind: SymbolKind.Method,
					location: {
						uri: currentDocUri,
						range: {
							start: doc.positionAt(rule.location[1]),
							end: doc.positionAt(rule.location[2])
						}
					}
				};
				result.push(symbolSummary);
			}

		});
	}
	// TODO
	console.log("The obtained symbol information is", result);
	// result.sort((rule1, rule2) => (rule2.location.range.start.line - rule1.location.range.start.line));
	return result;
});

// This handler provides the initial list of the completion items.
connection.onCompletion(
	(_textDocumentPosition: TextDocumentPositionParams): CompletionItem[] => {
		// The pass parameter contains the position of the text document in
		// which code complete got requested. For the example we ignore this
		// info and always provide the same completion items.
		return [
			// {
			// 	label: 'TypeScript',
			// 	kind: CompletionItemKind.Text,
			// 	data: 1
			// },
			// {
			// 	label: 'JavaScript',
			// 	kind: CompletionItemKind.Text,
			// 	data: 2
			// }
		];
	}
);

// This handler resolves additional information for the item selected in
// the completion list.
connection.onCompletionResolve(
	(item: CompletionItem): CompletionItem => {
		if (item.data === 1) {
			item.detail = 'TypeScript details';
			item.documentation = 'TypeScript documentation';
		} else if (item.data === 2) {
			item.detail = 'JavaScript details';
			item.documentation = 'JavaScript documentation';
		}
		return item;
	}
);

// Make the text document manager listen on the connection
// for open, change and close text document events
documents.listen(connection);

// Listen on the connection
connection.listen();

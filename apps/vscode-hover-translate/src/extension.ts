import * as vscode from "vscode";

type ExtensionConfig = {
  enabled: boolean;
  sourceLanguage: string;
  targetLanguage: string;
  maxSelectionLength: number;
  showOriginalText: boolean;
  provider: "google-web";
};

type TranslationResult = {
  translatedText: string;
  detectedSourceLanguage?: string;
};

const cache = new Map<string, TranslationResult>();

export function activate(context: vscode.ExtensionContext): void {
  const hoverProvider = vscode.languages.registerHoverProvider(
    { scheme: "*" },
    {
      provideHover: async (
        document: vscode.TextDocument,
        position: vscode.Position
      ): Promise<vscode.Hover | undefined> => {
        const editor = vscode.window.activeTextEditor;
        if (!editor || editor.document.uri.toString() !== document.uri.toString()) {
          return undefined;
        }

        const config = getConfig();
        if (!config.enabled) {
          return undefined;
        }

        const selection = editor.selections.find(
          (entry) => !entry.isEmpty && entry.contains(position)
        );
        if (!selection) {
          return undefined;
        }

        const selectedText = document.getText(selection).trim();
        if (!selectedText || selectedText.length > config.maxSelectionLength) {
          return undefined;
        }

        try {
          const result = await translateSelection(selectedText, config);
          return new vscode.Hover(
            buildHoverMarkdown(selectedText, result, config),
            selection
          );
        } catch (error) {
          const message =
            error instanceof Error ? error.message : "unknown translation error";
          return new vscode.Hover(
            new vscode.MarkdownString(
              `**Translation failed**\n\n${escapeMarkdown(message)}`
            ),
            selection
          );
        }
      }
    }
  );

  const clearCacheCommand = vscode.commands.registerCommand(
    "hoverTranslate.clearCache",
    () => {
      cache.clear();
      void vscode.window.showInformationMessage("Hover Translate cache cleared.");
    }
  );

  const translateSelectionOrClipboardCommand = vscode.commands.registerCommand(
    "hoverTranslate.translateSelectionOrClipboard",
    async () => {
      const config = getConfig();
      if (!config.enabled) {
        void vscode.window.showWarningMessage("Hover Translate is disabled.");
        return;
      }

      const input = await getSelectionOrClipboard(config.maxSelectionLength);
      if (!input) {
        void vscode.window.showWarningMessage(
          "No selected text or clipboard text is available for translation."
        );
        return;
      }

      await vscode.window.withProgress(
        {
          location: vscode.ProgressLocation.Notification,
          title: `Translating ${input.source} text`
        },
        async () => {
          const result = await translateSelection(input.text, config);
          await showTranslationDocument(input, result, config);
        }
      );
    }
  );

  context.subscriptions.push(
    hoverProvider,
    clearCacheCommand,
    translateSelectionOrClipboardCommand
  );
}

export function deactivate(): void {}

function getConfig(): ExtensionConfig {
  const config = vscode.workspace.getConfiguration("hoverTranslate");
  return {
    enabled: config.get<boolean>("enabled", true),
    sourceLanguage: config.get<string>("sourceLanguage", "auto"),
    targetLanguage: config.get<string>("targetLanguage", "zh-TW"),
    maxSelectionLength: config.get<number>("maxSelectionLength", 240),
    showOriginalText: config.get<boolean>("showOriginalText", true),
    provider: config.get<"google-web">("provider", "google-web")
  };
}

async function translateSelection(
  text: string,
  config: ExtensionConfig
): Promise<TranslationResult> {
  const cacheKey = JSON.stringify({
    provider: config.provider,
    sourceLanguage: config.sourceLanguage,
    targetLanguage: config.targetLanguage,
    text
  });
  const cached = cache.get(cacheKey);
  if (cached) {
    return cached;
  }

  const result = await translateWithGoogleWeb(
    text,
    config.sourceLanguage,
    config.targetLanguage
  );
  cache.set(cacheKey, result);
  return result;
}

async function translateWithGoogleWeb(
  text: string,
  sourceLanguage: string,
  targetLanguage: string
): Promise<TranslationResult> {
  const source = sourceLanguage === "auto" ? "auto" : sourceLanguage;
  const url =
    "https://translate.googleapis.com/translate_a/single" +
    `?client=gtx&sl=${encodeURIComponent(source)}` +
    `&tl=${encodeURIComponent(targetLanguage)}` +
    "&dt=t" +
    `&q=${encodeURIComponent(text)}`;

  const response = await fetch(url, {
    headers: {
      "User-Agent": "Mycel Hover Translate"
    }
  });

  if (!response.ok) {
    throw new Error(`translation request failed with status ${response.status}`);
  }

  const payload = (await response.json()) as unknown;
  if (!Array.isArray(payload) || !Array.isArray(payload[0])) {
    throw new Error("translation provider returned an unexpected payload");
  }

  const translatedText = payload[0]
    .map((entry) => {
      if (!Array.isArray(entry) || typeof entry[0] !== "string") {
        return "";
      }
      return entry[0];
    })
    .join("")
    .trim();

  if (!translatedText) {
    throw new Error("translation provider returned an empty translation");
  }

  const detectedSourceLanguage =
    typeof payload[2] === "string" ? payload[2] : undefined;

  return {
    translatedText,
    detectedSourceLanguage
  };
}

type TranslationInput = {
  text: string;
  source: "selection" | "clipboard";
};

async function getSelectionOrClipboard(
  maxSelectionLength: number
): Promise<TranslationInput | undefined> {
  const editor = vscode.window.activeTextEditor;
  const selectedText = editor?.selection.isEmpty
    ? ""
    : editor?.document.getText(editor.selection).trim() ?? "";
  if (selectedText) {
    if (selectedText.length > maxSelectionLength) {
      void vscode.window.showWarningMessage(
        `Selected text exceeds the ${maxSelectionLength} character limit.`
      );
      return undefined;
    }

    return {
      text: selectedText,
      source: "selection"
    };
  }

  const clipboardText = (await vscode.env.clipboard.readText()).trim();
  if (!clipboardText) {
    return undefined;
  }

  if (clipboardText.length > maxSelectionLength) {
    void vscode.window.showWarningMessage(
      `Clipboard text exceeds the ${maxSelectionLength} character limit.`
    );
    return undefined;
  }

  return {
    text: clipboardText,
    source: "clipboard"
  };
}

async function showTranslationDocument(
  input: TranslationInput,
  result: TranslationResult,
  config: ExtensionConfig
): Promise<void> {
  const detected = result.detectedSourceLanguage ?? config.sourceLanguage;
  const sourceLabel = input.source === "selection" ? "editor selection" : "clipboard";
  const content = [
    "# Translation",
    "",
    "## Result",
    "",
    result.translatedText,
    "",
    "## Source",
    "",
    "```text",
    input.text,
    "```",
    "",
    "## Meta",
    "",
    `- origin: ${sourceLabel}`,
    `- languages: ${detected} -> ${config.targetLanguage}`
  ].join("\n");

  const document = await vscode.workspace.openTextDocument({
    language: "markdown",
    content
  });
  await vscode.window.showTextDocument(document, {
    preview: true,
    viewColumn: vscode.ViewColumn.Beside
  });
}

function buildHoverMarkdown(
  selectedText: string,
  result: TranslationResult,
  config: ExtensionConfig
): vscode.MarkdownString {
  const markdown = new vscode.MarkdownString(undefined, true);
  markdown.isTrusted = false;
  markdown.supportHtml = false;

  markdown.appendMarkdown("**Translation**\n\n");
  markdown.appendMarkdown(`${escapeMarkdown(result.translatedText)}\n\n`);

  if (config.showOriginalText) {
    markdown.appendMarkdown("---\n\n");
    markdown.appendMarkdown(`**Source**: ${escapeMarkdown(selectedText)}\n\n`);
  }

  const detected = result.detectedSourceLanguage ?? config.sourceLanguage;
  markdown.appendMarkdown(
    `$(globe) ${escapeMarkdown(detected)} -> ${escapeMarkdown(
      config.targetLanguage
    )}`
  );

  return markdown;
}

function escapeMarkdown(value: string): string {
  return value.replace(/[\\`*_{}[\]()#+\-.!|>]/g, "\\$&");
}

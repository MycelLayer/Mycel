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

  context.subscriptions.push(hoverProvider, clearCacheCommand);
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

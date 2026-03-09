# Mycel Hover Translate

Minimal VS Code extension for this workflow:

1. select text
2. move the mouse over the selected text
3. wait for the editor hover
4. see a floating translation popup above the selection

## Current Behavior

- only triggers when the current selection is non-empty
- only triggers when the hover position is inside the current selection
- translates the exact selected text
- caches repeated translations in memory
- defaults to `auto -> zh-TW`
- currently uses the public Google web translate endpoint shape, so network access is required

## Settings

- `hoverTranslate.enabled`
- `hoverTranslate.sourceLanguage`
- `hoverTranslate.targetLanguage`
- `hoverTranslate.maxSelectionLength`
- `hoverTranslate.showOriginalText`
- `hoverTranslate.provider`

## Development

```bash
npm install
npm run compile
```

Then open this folder in VS Code and press `F5` to launch the extension host.

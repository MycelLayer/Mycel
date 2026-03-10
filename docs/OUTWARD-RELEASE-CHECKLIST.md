# Outward Release Checklist

Use this checklist after changing any public-facing project surface such as:

- GitHub repo description, topics, homepage, or social preview
- `README.md` or `README.zh-TW.md`
- `docs/index.html`
- `docs/social-preview.svg`
- `docs/social-preview.png`

This checklist is intentionally narrow. It is for outward presentation consistency, not protocol or implementation release validation.

## 1. Repo Host

- [ ] GitHub repo `description` still matches the current project positioning.
- [ ] GitHub repo `homepage` points to the intended public landing page.
- [ ] GitHub repo `topics` still reflect the current technical scope.
- [ ] GitHub Discussions remains enabled if we still want public design conversation.
- [ ] GitHub repo social preview image still looks correct in the repo settings UI.

## 2. README

- [ ] [`README.md`](../README.md) opens with the current one-sentence positioning.
- [ ] [`README.zh-TW.md`](../README.zh-TW.md) stays aligned with the English landing message.
- [ ] The quickstart commands still exist and still demonstrate the intended first impression.
- [ ] The "What it is not" section still matches current positioning.
- [ ] Key document links still resolve.

## 3. Homepage

- [ ] [`docs/index.html`](./index.html) still reflects the same positioning as the README.
- [ ] The main CTA links still point to the intended GitHub and spec pages.
- [ ] `og:title`, `og:description`, and `og:image` still match the current landing message.
- [ ] `twitter:image` still points to the intended social preview image.
- [ ] The page still loads from `https://ctf2090.github.io/Mycel/`.

## 4. Social Preview

- [ ] [`docs/social-preview.svg`](./social-preview.svg) is the editable source of truth.
- [ ] [`docs/social-preview.png`](./social-preview.png) has been regenerated after SVG edits.
- [ ] The PNG remains `1200x630`.
- [ ] Text fits safely inside the frame at normal preview size.
- [ ] The GitHub repo preview and Pages preview are visually acceptable after cache refresh.

## 5. Public Consistency Check

- [ ] Repo description, README first paragraph, and homepage hero say the same core thing.
- [ ] Repo share card and Pages share card do not contradict each other.
- [ ] Links shared to GitHub should preview the repo card we expect.
- [ ] Links shared to `https://ctf2090.github.io/Mycel/` should preview the Pages card we expect.

## 6. Post-publish Verification

- [ ] Latest `CI` workflow completed successfully after the outward-facing change.
- [ ] Latest `pages-build-deployment` workflow completed successfully if homepage assets changed.
- [ ] `curl -I https://ctf2090.github.io/Mycel/` returns `200`.
- [ ] `curl -I https://ctf2090.github.io/Mycel/social-preview.png` returns `200` when that asset changed.
- [ ] GitHub Community profile remains healthy enough for public contribution entry.

## 7. Optional Commands

These commands are useful for a quick re-check:

```bash
gh run list -R ctf2090/Mycel --limit 5
gh api repos/ctf2090/Mycel --jq '{description:.description,homepage:.homepage,topics:.topics,has_discussions:.has_discussions,has_pages:.has_pages}'
gh api repos/ctf2090/Mycel/community/profile
curl -I -L https://ctf2090.github.io/Mycel/
curl -I https://ctf2090.github.io/Mycel/social-preview.png
curl -s https://github.com/ctf2090/Mycel | rg 'og:image|og:title|og:description'
curl -s https://ctf2090.github.io/Mycel/ | rg 'og:image|og:title|og:description|twitter:image'
```

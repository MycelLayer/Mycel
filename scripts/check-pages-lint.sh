#!/usr/bin/env bash
set -euo pipefail

if git diff --cached --quiet -- pages/; then
  exit 0
fi

echo "pages/ changes detected; running local Pages lint..."
npm run lint:pages

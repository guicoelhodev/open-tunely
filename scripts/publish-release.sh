#!/usr/bin/env bash
# Publish a single public rolling release after both desktop builds pass.
set -euo pipefail
: "${GH_REPO:?GH_REPO is required}"
: "${COMMIT:?COMMIT is required}"
release_tag=$COMMIT
[[ "$release_tag" =~ ^v?[0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.-]+)?$ ]] || { echo 'Expected an existing version tag.' >&2; exit 1; }
source_commit=$(git rev-parse "refs/tags/$release_tag^{commit}")
artifacts=$(realpath "${1:-dist}")
for name in Open-Tunely-linux-x64.AppImage Open-Tunely-windows-x64-setup.exe; do
  test -s "$artifacts/$name"
done
(
  cd "$artifacts"
  sha256sum Open-Tunely-linux-x64.AppImage Open-Tunely-windows-x64-setup.exe > SHA256SUMS.txt
)
notes=$(mktemp)
trap 'rm -f "$notes"' EXIT
cat > "$notes" <<NOTES
Download Open Tunely for Windows or Linux below. These files are public and do not require a GitHub account.

- **Windows x64:** Open-Tunely-windows-x64-setup.exe
- **Linux x64:** Open-Tunely-linux-x64.AppImage
- **Verify downloads:** SHA256SUMS.txt

On Linux, mark the AppImage executable before opening it. The Windows installer is unsigned and downloads WebView2 if needed. Download only content you own or have permission to save.

This release is updated after successful builds from the selected source revision.

Source commit: $source_commit
NOTES
if ! gh release view "$release_tag" > /dev/null 2>&1; then
  gh release create "$release_tag" --verify-tag --draft \
    --title "Open Tunely $release_tag" --notes-file "$notes"
fi
# Upload both packages first, then the manifest identifying the complete set.
gh release upload "$release_tag" \
  "$artifacts/Open-Tunely-linux-x64.AppImage" \
  "$artifacts/Open-Tunely-windows-x64-setup.exe" --clobber
gh release upload "$release_tag" "$artifacts/SHA256SUMS.txt" --clobber
gh release edit "$release_tag" --draft=false --prerelease=false --latest \
  --title "Open Tunely $release_tag" --notes-file "$notes"
# Retire the superseded private test draft only after public downloads exist.
if [ "$(gh release view test-42234798019-1 --json isDraft --jq .isDraft 2>/dev/null || true)" = true ]; then
  gh release delete test-42234798019-1 --yes
fi

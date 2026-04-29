# Releasing mediacast-netcatalog

How to cut a new release. Once tokens are configured (one-time, below),
each release is **one tag push**.

## One-time setup

Two registry tokens need to live in this repo's GitHub Actions secrets:

| Secret name | Where to get it | Used by |
|---|---|---|
| `CARGO_REGISTRY_TOKEN` | https://crates.io/me — "API Tokens" → New Token (scope: `publish-update`) | `cargo publish` job in `release.yml` |
| `PYPI_API_TOKEN` | https://pypi.org/manage/account/token/ — "Add API token" (scope: this project once it exists, or "all projects" for the first publish) | `maturin upload` job in `release.yml` |

To add them:

```
gh secret set CARGO_REGISTRY_TOKEN --repo Mediacastnet/mediacast-netcatalog
gh secret set PYPI_API_TOKEN       --repo Mediacastnet/mediacast-netcatalog
```

Or via the GitHub UI: Settings → Secrets and variables → Actions →
New repository secret.

### PyPI environment (recommended)

The release workflow's `pypi-publish` job is gated on a GitHub Actions
**environment** named `pypi`. This adds a layer of safety: you can
configure required reviewers + branch restrictions on the environment.

Set up via Settings → Environments → New environment → name `pypi`. Add
required reviewers if you want a human gate before each PyPI push.

If you'd rather skip the environment gate, edit `.github/workflows/release.yml`
and remove the `environment: pypi` line on the `pypi-publish` job.

### First-publish caveats

**crates.io**:
- Crate name `mediacast-netcatalog` is reserved on first publish; subsequent
  publishes from a different account fail.
- You must claim the name from an account that's a member of (or owner of)
  the GitHub `Mediacastnet` org's crates.io equivalent. Add other org
  maintainers as crate owners post-publish: `cargo owner --add github:mediacastnet:maintainers mediacast-netcatalog`.

**PyPI**:
- Same name-reservation rule. Once `mediacast-netcatalog` is published, the
  name is owned by the publishing account.
- Consider [PyPI Trusted Publishing](https://docs.pypi.org/trusted-publishers/)
  as a more secure alternative to long-lived API tokens. To switch:
  - Configure trusted publishing on PyPI for `Mediacastnet/mediacast-netcatalog`
  - Drop `PYPI_API_TOKEN` and use OIDC instead (the maturin-action supports it
    via `args: --skip-existing dist/*` plus removing the `env:` block)
  - Add `permissions: { id-token: write }` to the `pypi-publish` job

## Cutting a release

1. **Bump the version** in two files. They must agree (the `verify` job
   in `release.yml` enforces this, with `0.2.0.dev0` ↔ `0.2.0-dev`
   normalization for PEP 440 / SemVer differences):
   - `Cargo.toml`: `version = "0.2.0"`
   - `pyproject.toml`: `version = "0.2.0"`

2. **Update `CHANGELOG.md`**. Move the `[Unreleased]` heading to
   `[X.Y.Z] — YYYY-MM-DD`, add a new empty `[Unreleased]` section above.
   The release workflow extracts the matching section and posts it as
   the GitHub Release notes.

3. **Commit** the version bump + changelog as one commit:
   ```
   git add Cargo.toml pyproject.toml CHANGELOG.md
   git commit -m "chore(release): 0.2.0"
   ```

4. **Tag** with a `v`-prefixed SemVer string. The tag name minus `v` must
   match the Cargo version exactly:
   ```
   git tag v0.2.0
   git push origin main v0.2.0
   ```

5. **Watch the workflow run** at `https://github.com/Mediacastnet/mediacast-netcatalog/actions`.
   The pipeline:
   - Verifies tag ↔ Cargo.toml ↔ pyproject.toml versions agree
   - Runs the test suite one final time
   - Publishes to crates.io
   - Builds wheels for Linux x86_64 + aarch64, macOS x86_64 + arm64, Windows x86_64
   - Builds an sdist
   - Publishes wheels + sdist to PyPI
   - Creates a GitHub Release with the changelog section attached

   If anything fails, the tag stays in place. Fix the issue, force-push
   the tag if needed (`git tag -d v0.2.0 && git tag v0.2.0 && git push -f origin v0.2.0`),
   or cut a `v0.2.1` patch.

## Pre-releases

Tags with `-rc`, `-alpha`, `-beta`, or `-dev` suffixes are flagged as
GitHub pre-releases automatically. Pre-releases are useful for letting
NetCaster pin a specific version before cutting `0.2.0` final:

```
git tag v0.2.0-rc.1
git push origin v0.2.0-rc.1
```

The workflow publishes pre-release versions to crates.io and PyPI just
the same; consumers must opt in via explicit version pins
(`mediacast-netcatalog = "=0.2.0-rc.1"` in `Cargo.toml`).

## Yanking a bad release

If a published version has a critical bug:

```bash
# crates.io
cargo yank --version 0.2.0

# PyPI doesn't support yanking quite the same way; mark it as removed:
# https://pypi.org/manage/project/mediacast-netcatalog/release/0.2.0/
# (use the "Yank release" button)
```

Then cut a `0.2.1` with the fix immediately. Yanking doesn't remove the
version; it prevents new pins from picking it up.

## Manual fallback

If the workflow is broken and you need to publish manually:

```bash
# Crates.io
cargo publish --token "$CARGO_REGISTRY_TOKEN"

# PyPI (from a clean checkout)
pip install "maturin>=1.5,<2.0"
maturin build --release --features python --out dist
maturin upload --repository pypi dist/* --token "$PYPI_API_TOKEN"
```

This skips the cross-platform wheel matrix — you get only the wheel for
your local platform. Acceptable as a one-time emergency; restore the
automated pipeline as soon as the underlying issue is fixed.

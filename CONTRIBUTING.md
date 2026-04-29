# Contributing to mediacast-netcatalog

Thanks for your interest. This project is **research-grade data + a Rust
crate** maintained by [Mediacast Network Solutions](https://github.com/Mediacastnet).
First production consumer is our own [NetCaster](https://github.com/Mediacastnet/netcaster);
that biases what we accept, but the project is genuinely open and high-value
contributions from outside the venue-network space are welcome.

## What we want

### Catalog contributions (highest value)

- **New vendor catalogs**. Extreme, FortiSwitch, MikroTik, Brocade/RUCKUS,
  Dell EMC, Edgecore, Cumulus/SONiC. Aim for ≥80% of the 25 abstract command
  types. Cite vendor docs in the `sources:` block.
- **Firmware-version drift entries** for vendors already covered. If you
  hit an output-format change between firmware revisions on a vendor we
  ship, send a PR adding a `versions:` block + sample output.
- **Real-gear validation**. Take an entry marked `unverified: true`, run it
  against actual hardware, capture the real output, and PR a fix that
  removes the flag.

### Crate contributions

- **Probe protocol additions**. v0.1 stubs NETCONF/gNMI/RESTCONF/SSH;
  IPMI, ONIE, SONiC discovery would round it out.
- **Version-matcher edge cases**. If your vendor's firmware string format
  doesn't parse cleanly, send a failing test + the fix.
- **Performance**. Catalog loading is `O(files)`, lookup is `O(versions
  blocks)`. If you're driving high volume and have a profile, send a PR.

### What we don't want

- Drive-by formatting or rename PRs.
- New abstract command types without a real-world consumer requesting them.
  The 25 we have were extracted from a working codebase; new ones need the
  same provenance.
- Refactors that don't fix anything.

## Catalog YAML conventions

See `catalog/SCHEMA.md` for the full spec. Quick rules:

- Every entry **must** cite its source. URL + access date in the
  `sources:` block, or per-version `notes:` if the source is the same as
  another entry.
- `cli` is a single line. Multi-line composite commands (Junos `configure
  → set → commit`) use a multi-line YAML string with one CLI command per
  line.
- `applies_to` is a SemVer-flavored range. See the schema doc for the
  recognized syntax.
- For commands the vendor genuinely doesn't expose, set `cli:
  "NOT_SUPPORTED"` and document why in `notes`.
- Mark `unverified: true` if the data was extracted from heuristic /
  community sources rather than vendor docs.

## Development setup

```bash
git clone https://github.com/Mediacastnet/mediacast-netcatalog
cd mediacast-netcatalog

# Rust
cargo test --no-default-features
cargo run --example basic_lookup

# Python bindings
pip install "maturin>=1.5,<2.0"
maturin develop --features python
python -c "from mediacast_netcatalog import Catalog; print(Catalog.load_bundled().vendors())"
```

## PR process

1. Open an issue first for substantive changes (new vendor, schema
   change, breaking API). For obvious fixes, just send the PR.
2. Run `cargo fmt`, `cargo clippy`, `cargo test`, and `yamllint catalog/`
   before submitting.
3. Add an entry to `CHANGELOG.md` under `[Unreleased]`.
4. CI must pass on Linux + macOS + Windows.

## Releasing

See [`RELEASING.md`](RELEASING.md) for the release process. Maintainers
cut releases by tagging `vX.Y.Z` on `main`; the GitHub Actions workflow
handles `cargo publish`, multi-platform wheel builds + PyPI upload, and
the GitHub Release announcement automatically.

## License

By contributing, you agree your contribution is dual-licensed under
MIT or Apache-2.0, matching the rest of the project. See `LICENSE-MIT`
and `LICENSE-APACHE`.

## Code of conduct

Be kind. Disagree on technical merit, not on people. Maintainers will
moderate at their discretion.

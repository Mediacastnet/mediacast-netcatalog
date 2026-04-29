# Vendor Command Catalog — Cross-vendor Findings (2026-04-28)

Synthesis of the 2026-04-28 doc-crawl research effort across seven
vendors (Cisco IOS-XE, Cisco NX-OS, Aruba AOS-CX, Juniper Junos,
Arista EOS, HPE ProCurve, Cisco Meraki MS). The per-vendor YAML
files are the data layer; this doc captures the patterns that show
up when you read all seven side-by-side.

Audience: future implementers building the runtime catalog loader
(per the vendor-coverage roadmap entry) and anyone thinking about
NetCaster's vendor-coverage architecture.

---

## 1. The catalog is uniformly populatable

**All 7 vendors covered all 25 abstract command types**, with only
4 entries (out of 175 vendor-command pairs) marked `NOT_SUPPORTED`.
This is a **positive finding for the architectural direction**: the
abstract command vocabulary in
[`COMMAND_TYPES.md`](COMMAND_TYPES.md) is genuinely vendor-neutral.
We didn't have to bolt on vendor-specific primitives.

The 4 `NOT_SUPPORTED` entries:

| Vendor | Command | Reason |
|---|---|---|
| Arista EOS | `CIVIC_LOCATION` | LLDP-MED Network Policy + basic location TLVs only; no first-class civic-address config tree |
| HPE ProCurve | `CIVIC_LOCATION` | Same — civic TLV is AOS-CX-and-newer territory |
| Cisco Meraki MS | `CIVIC_LOCATION` | No LLDP-MED civic exposure |
| Cisco Meraki MS | `SAVE_CONFIG` | Cloud-managed: no running/startup distinction |

`CIVIC_LOCATION` is a NetCaster-specific need driven by Aruba AOS-CX
support; relax it to "best-effort" in the runtime catalog and accept
that not every vendor provides it.

---

## 2. The original protocol triad (NETCONF/RESTCONF/gNMI) was incomplete

The schema's original protocol-alternative slots assumed a Cisco-/
OpenConfig-shaped world. Three of the seven vendors required
extensions:

- **Arista EOS** — eAPI (JSON-RPC over HTTPS) is the flagship
  programmatic interface alongside gNMI. Older than gNMI, more
  widely deployed.
- **HPE ProCurve** — proprietary REST API + SNMP are the only
  programmatic surfaces. NETCONF/RESTCONF/gNMI **never delivered**
  on this line. Aruba's protocol investment landed exclusively
  on AOS-CX.
- **Cisco Meraki MS** — Dashboard API (cloud REST + JSON) is the
  whole game. Local CLI is diagnostic-only.

[`SCHEMA.md`](SCHEMA.md) is updated to bless these extensions
formally. Lesson: the protocol-alternative slot list is not closed.

---

## 3. "Cisco-IOS-compatible" is a real category — but the divergences bite

Three vendors — Cisco IOS-XE, Cisco NX-OS, Arista EOS — share much
of the *user-level* CLI surface (`show arp`, `show mac
address-table`, `shutdown`/`no shutdown`, `switchport access vlan`,
`write memory` / `copy running-config startup-config`).

**This is misleading.** A parser tuned for Cisco IOS-XE will silently
produce wrong results on NX-OS or EOS in load-bearing ways. From the
research:

### NX-OS divergences from IOS-XE that would corrupt parsing

- **DHCP relay syntax**: NX-OS uses `ip dhcp relay address`, not
  `ip helper-address`. Existing `parse_svi_config` (inherited from
  the IOS handler in NetCaster's current codebase) **misses DHCP
  servers on NX-OS entirely**.
- **Portfast**: `spanning-tree port type edge`, not `spanning-tree
  portfast`.
- **LLDP labels**: NX-OS emits `Local Port id:`, IOS uses `Local
  Intf:` — different field names break neighbor parsing.
- **IP address format**: NX-OS emits CIDR (`10.1.10.1/24`); IOS
  emits dotted-mask. Existing parser only matches dotted-mask.
- **`show interface` status format**: NX-OS splits oper/admin onto
  two lines; IOS regex won't match.
- **Modular vs stack**: Nexus chassis serial model differs from
  Catalyst stack model. Stack parsing assumptions won't apply.

### EOS divergences from IOS-XE

- **Interface naming**: `Et24` / `Ethernet24` (no `1/0/24`).
- **SVI IP**: CIDR, not dotted-mask.
- **STP per-VLAN**: `spanning-tree vlan-id <X>` (with `-id`).
- **PoE**: `poe disabled`, not `power inline never`.
- **Default STP mode**: MSTP, not PVST+.
- **Default L2 MTU**: 9214, not 1500.

**Implication for the runtime catalog loader:** the per-vendor split
matters even when CLI strings look identical. Parsers must dispatch
on `vendor` (and ideally `firmware-version-range`) before regexing.

---

## 4. Junos's transactional config model is a different category entirely

Junos isn't "another flavor of Cisco-CLI." Every write is:

```
configure
set <hierarchical config path> <value>
commit
```

Single-line imperative forms (`shutdown`, `switchport access vlan
X`, `write memory`) don't exist. The catalog's `cli` field for
Junos write commands is **multi-line by design** — flattening to
single lines yields invalid Junos.

Specific divergences worth highlighting for an implementer:

- **`commit` IS save.** SAVE_CONFIG is implicit in any committed
  write. Calling SAVE_CONFIG separately is a no-op; emitting `write
  memory` is a syntax error.
- **STP "portfast + bpduguard" is split** across two hierarchies
  (per-port `set protocols rstp interface X edge` + global `set
  protocols rstp bpdu-block-on-edge`). No per-port BPDU guard
  shortcut.
- **VLAN assignment is replace-not-append.** Must `delete` existing
  members before `set`-ing the new one, otherwise the port becomes
  a multi-VLAN access port (invalid state).
- **SVI binding is inverted.** IRB unit doesn't declare its VLAN;
  the VLAN declares its `l3-interface irb.N`. Two config trees
  touched per SVI_CREATE.
- **`commit confirmed <minutes>` auto-rollback.** Junos has built-in
  audit-grade safety: a commit that isn't re-confirmed within the
  window auto-reverts. **Worth adopting platform-wide as a pattern
  even on non-Junos vendors** (NetCaster could implement an
  equivalent safety net via "save current config, push change, wait
  N minutes for operator confirm, else restore").

**Implication for the runtime catalog loader:** the abstraction
needs to handle multi-line commit-style writes, not just imperative
single-line forms. The current `SwitchPlatform` ABC's
`get_*_commands` returning `list[str]` is sufficient mechanically;
the higher-level engine code that calls `send_config_set` may not be.

---

## 5. Cloud-managed (Meraki) is a different product shape

The Meraki research returned a substantive "should this even live
in this catalog?" recommendation. Six load-bearing reasons NOT to
shoehorn Meraki MS into the existing `SwitchPlatform` model:

1. **Auth model**: org-scoped Bearer token, not per-device
   credentials → incompatible with NetCaster's per-switch
   Fernet-encrypted credential record.
2. **Rate limit**: aggregate per-org (10 rps), not per-device →
   concurrent-discovery patterns will throttle.
3. **Operation scope**: many ops are network-/org-scoped (LAGs,
   VLANs, STP, VLAN profiles), not switch-scoped → venue-to-switch
   mapping is wrong shape.
4. **Reads**: typed JSON, not text-parsed CLI → existing parsers
   don't apply.
5. **Writes**: atomic and persistent with no save step, no SSH
   config-mode session, async failure modes (Live Tools).
6. **No firmware version**: rolling cloud updates → the catalog's
   version-aware-selection premise is moot.

**Recommended action:** treat Meraki as a parallel `CloudManagedPlatform`
abstraction, not as another `SwitchPlatform` subclass. Keep the
catalog YAML as research/reference, but don't expect a runtime catalog
loader to drive a Meraki integration through the existing per-command-
string dispatch path.

This deserves a proposing ADR before any Meraki implementation work.

---

## 6. Firmware-version awareness is the real win

The existing `SwitchPlatform` handlers in
`netcaster_engine/platforms/` are **firmware-version-blind**. The
research surfaced concrete cases where this matters:

- **Cisco IOS-XE 12.x vs 15.x vs 17.x**: `show ip arp` column count
  differs; `show vlan brief` reserved-VLAN status format differs;
  `show mac address-table` vs `show mac-address-table` (note hyphen).
- **NX-OS protocol introduction**: NETCONF/RESTCONF added in 9.2(1);
  gNMI Get/Set OpenConfig in 9.3(5). A protocol-aware engine needs
  to know.
- **Aruba AOS-CX**: NETCONF effectively `null`; gNMI `>=10.17`;
  REST API versioned `v10.04` → `v10.16`. Family-prefixed firmware
  strings (`FL.10.13.1000`) require version-matcher special-casing.
- **Cisco IOS-XE programmability**: NETCONF + RESTCONF in 16.6,
  gNMI in 16.10. The gap matters for v1 NetBox-imported switches
  on older firmware.
- **EOS gNMI paths**: different per release train; documented but
  not statically queryable.

**The runtime catalog's version-matcher logic is load-bearing.** A
straight "match exact version" approach will miss most real-world
deployments (operators run a mix). The matcher needs:

- Range support (`>=15.0,<17.0`).
- Most-specific-wins precedence.
- Family-prefix awareness for Aruba AOS-CX (strip `FL.`/`GL.`/`LL.`/
  `ML.`/`DL.`/`PL.`/`QL.` before SemVer compare).
- A defined fallback when no exact-version match exists (use the
  most recent older version's entry? error and surface to operator?).

The proposing ADR for the runtime catalog should specify these.

---

## 7. Doc accessibility was a real cost on three vendors

WebFetch was 403-blocked on:

- **Aruba AOS-CX** — every `arubanetworking.hpe.com/techdocs/...`
  URL.
- **HPE ProCurve** — same HPE doc portal.
- **Some Cisco command-reference URLs** — the agent fell back to
  DevNet / GitHub mirror sources.

Mitigation worked: agents fell back to community sources, vendor
GitHub repos (YangModels/yang, aruba/aoscx-yang), and Aruba's
asp.arubanetworks.com community pages. Quality marked accordingly
in the YAML files (`unverified` flag where source confidence is
lower).

**Implication for catalog maintenance:** when re-running this
research (e.g., to capture new firmware), the doc-access friction
won't go away. Plan for it. Possibly: cache vendor docs locally as
checked-in PDFs / HTML snapshots when accessible.

---

## 8. What's next

This research output is the **data layer** of the vendor-coverage
architecture rethink. It's not yet wired into NetCaster's runtime.
The natural next artifacts:

1. **A proposing ADR** (likely ADR-0019 once decided) that:
   - Defines the runtime catalog-loader API.
   - Specifies firmware-version matching logic (with the
     family-prefix special case captured here).
   - Specifies the migration plan: convert
     `netcaster_engine/platforms/` handlers to consume the catalog
     rather than hard-coding commands.
   - Specifies how the catalog composes with
     [ADR-0018](../decisions/0018-hybrid-protocol-read-path-investigation.md)
     (per-command CLI vs protocol selection).
   - Decides whether Meraki (and any future cloud-managed vendor)
     lives in this catalog or a parallel `CloudManagedPlatform` one.

2. **Catalog file maintenance.** New firmware releases change the
   facts. Annual re-eval cadence — track under tech-debt #13's
   pattern (next: 2027-04).

3. **Validate against real gear.** When the runtime catalog is
   wired, validate against the test-bench venues (MFP / Yankees /
   Bills) before customer rollout. Especially for the divergences
   flagged in §3 above.

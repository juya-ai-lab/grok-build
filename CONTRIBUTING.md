# Contributing

This repository does **not** accept external pull requests or unsolicited
patches.

SpaceXAI develops this software internally. The public tree is published for
source transparency and local builds under the terms of the Apache License,
Version 2.0 (see [`LICENSE`](LICENSE)).

## Bug reports and compatibility issues

Use the repository's GitHub issue form for reproducible bugs and compatibility
reports. Select the applicable scope (`upstream-originated`, `fork-specific`,
or `both`) and record the upstream and fork version/tag, commit, commit URL,
and validation status separately. The GitHub issue is the canonical record;
keep it open until the stated artifact-level validation and other close
criteria are complete.

## Issue labels

The bug template applies the existing `bug` label. Maintainers add scope and
status labels after checking the evidence:

| Label | Use when |
|---|---|
| `upstream-originated` | The issue reproduces in an upstream version or baseline and its root cause is upstream-originated. |
| `upstream-conflict` | Fork behavior or integration conflicts with upstream behavior or changes; this requires an actual divergence or merge/integration conflict, not merely an upstream bug also present in the fork. |
| `fork-specific` | Only this fork's changes, privacy trims, or release process are affected. |
| `needs-artifact-validation` | Source checks pass, but a released or packaged artifact still needs validation. |

`upstream-originated` and `upstream-conflict` describe different evidence and
must not be used interchangeably. A scope label can be combined with the
artifact-validation status label.

## Security reports

Please report security issues through the process described in
[`SECURITY.md`](SECURITY.md). Do not open a public issue for vulnerabilities.

## Licensing of this source

By downloading or using this source, you agree that your use is governed by
the Apache License, Version 2.0. No contributor license agreement is offered
because external contributions are not accepted.

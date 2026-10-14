# Security policy

[Português brasileiro](docs/pt-BR/contributors/security.md)

## Current status and reporting

There are no published supported releases yet. Phase 2 provides a runnable Rust daemon/diagnostic client and implements bounded local IPC, same-user authorization and reviewed HID policy boundaries. Linux software/runtime checks and Windows static checks are separate from a security audit or hardware/Windows runtime validation; see [implementation evidence](docs/architecture/implementation-status.md). Future desktop/drivers/research tooling remain governed by this policy.

A maintainer-verified private reporting contact **has not been configured**. The canonical repository is [ExtremeCodeX/peripheral-hub](https://github.com/ExtremeCodeX/peripheral-hub); availability of private vulnerability reporting has not been verified. Before accepting sensitive reports or publishing releases, maintainers must provide and test a private channel and establish response ownership. No response-time commitment or email address is invented here.

Once GitHub private vulnerability reporting is enabled on the canonical repository, use **Security → Report a vulnerability**. Otherwise, request a private reporting channel from the maintainer without disclosing technical details publicly. Do not post exploit instructions, credentials, full captures, vendor binaries, or identifiable device data in issues or PRs. Until a channel is verified, keep sensitive details local.

A private report should include the affected revision/component, impact, preconditions, sanitized reproduction, relevant platform/firmware, and whether hardware writes or elevated permissions are involved. Share only the minimum necessary evidence through the verified channel. Coordinate disclosure with maintainers; do not assume a release or fix is already available.

## Device and application boundaries

- The daemon belongs to the user's session, with no automatic administrative privileges. IPC must be local, explicitly versioned, authenticated/restricted to the same user, and never exposed on a public interface.
- Tauri permissions must be minimal; the frontend must not send arbitrary HID/USB operations or bypass daemon capability checks.
- Drivers and community manifests are untrusted inputs. Validate identities, schema versions, types, byte lengths, parameter ranges, sequencing, timeouts, and rate limits before modeled operations.
- Native Rust drivers are reviewed and compiled into the project. No automatically loaded third-party DLLs or arbitrary native plugins in the MVP.
- Firmware flashing, bootloader entry, EEPROM writes, undocumented persistent writes, and destructive/unknown commands are prohibited in this scope. A skill cannot authorize them.
- Discovery must not write arbitrary packets. Stop on uncertain safety, unexpected traffic, disconnection, or ambiguous device/interface matching.
- Initial macro support is safe editing/validation/persistence/simulation; global hooks and input injection require a separately authorized and tested scope.

## Responsible research and consent

Research only software and hardware you own or are authorized to examine, subject to applicable law and license terms. Record source, authorization, device identity, research goal, and consent locally before capturing or performing experiments.

Never automatically run a vendor installer or unknown executable. Dynamic analysis needs explicit authorization and an isolated laboratory machine/VM where appropriate. VM USB passthrough and physical hardware constraints may limit analysis; do not infer safety from virtualization alone. Elevated USBPcap capture, when needed, is an explicit laboratory action, not a daemon privilege requirement.

REA is optional development tooling, not part of the runtime. Verify actual tool installation/startup instructions before configuring MCP; never commit tokens or guessed endpoints. Do not upload binaries, captures, or research data to external tools/services without explicit authorization and data review.

## Research data and privacy

Keep raw material in the Git-ignored repository-root `local-data/` directory or another verified ignored location. Suggested local organization: `local-data/vendor/`, `local-data/captures/`, `local-data/sessions/`, and `local-data/reports/`. These are local conventions, not directories created or commands implemented by this stage.

USB captures can include traffic from unrelated devices and typed input. Filter by target device/interface, capture the shortest necessary period, avoid entering personal information, and sanitize before export. Remove or replace serial numbers, private paths, usernames, keys, input contents, unrelated traffic, and other identifiers. Logs should omit sensitive raw bytes by default.

Do not redistribute vendor installers, executables, DLLs, firmware, keys, raw captures, or protected content. Only reviewed, necessary, authorized synthetic or sanitized fixtures may be committed. Label synthetic data explicitly and retain non-sensitive provenance and uncertainty. Ignore rules cannot prevent explicit force-adds or protect already tracked files; review every staged diff and artifact before sharing.

Do not store proprietary binaries or sensitive captures in SQLite, distributed packages, CI artifacts, or public research reports. Set retention and deletion expectations for local evidence with the researcher; avoid unrequested telemetry.

## Validation and release expectations

Treat simulation, static inference, software testing, and physical validation as distinct evidence. Record capability-specific platform and firmware coverage. Hardware tests are manual/opt-in, never automatically run by CI.

Review dependencies, Tauri capabilities, IPC authorization, logs, parsers, manifests, and export sanitization before release. Signing certificates, publishing credentials, private reporting ownership, and license confirmation are pending. Automated releases or public data sharing require explicit maintainer authorization.

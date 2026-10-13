# Writing a device protocol specification

This document defines a documentation format, not a device packet protocol or manifest schema. Never fill gaps with invented packets or identifier values.

## Required sections in `protocol.md`

1. **Identity and scope:** observed manufacturer/model, VID/PID, interface/usage, transport, firmware/revision, and capabilities under investigation. Explain distinguishing evidence and ambiguity.
2. **Provenance:** authorized vendor software version/hash, tools and versions, research dates, sanitized evidence references, and capture action timeline. Keep private source paths and consent details local.
3. **Lifecycle:** discovery versus opening, required initialization, safe state transitions, closing, disconnect behavior, and known revision differences.
4. **Report framing:** only evidenced transfer types, report IDs, lengths, byte offsets/meaning, endianness, sequencing, checksums, ACKs, and timing. Mark unknown fields unknown.
5. **Operations:** purpose, supported parameters, bounds, evidence, volatility/persistence, preconditions, response/error handling, and confidence for each operation. Unknown persistent effects prohibit experimentation in this scope.
6. **Uncertainty and exclusions:** competing hypotheses, unverified firmware/platforms, unsupported features, unsafe/unknown operations that must not be sent.
7. **Validation references:** point to software fixtures/tests and `validation.md` without treating static findings or synthetic data as physical proof.

## Evidence vocabulary

Protocol hypotheses use `confirmed-on-hardware`, `inferred`, `unverified`. Support/validation claims use `simulated`, `inferred`, `tested`, `validated-on-hardware`. Each label must include its basis and scope; these are not universal compatibility badges.

Synthetic fixtures should be deterministic, textual, and conspicuously labeled. Sanitized fixtures must preserve relevant semantics without identifiable payloads or unrelated traffic. Record redactions and limitations. Neither fixture category should contain vendor binaries, protected dumps, or full raw captures.

## Validation record

`validation.md` should state capability, driver revision, OS, firmware/device revision, interface, test method, actual expected/observed result, evidence label, date, and limitations. Document failures and unavailable hardware explicitly. A capability validated on one revision must not imply support on others.

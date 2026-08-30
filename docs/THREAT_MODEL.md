# Threat model

Sombra is designed for community connectivity during outages, disasters and areas with unreliable infrastructure.

## Protect

- message confidentiality once cryptographic envelopes are implemented
- message authenticity
- integrity of stored bundles once authenticated envelopes are implemented
- resilience when individual links or nodes disappear
- bounded resource use under duplicate or retry pressure
- unnecessary exposure of social-graph metadata

## Assume

- infrastructure may be unavailable for long periods
- peers may move in and out of contact
- some relays may be lost or untrusted
- radio conditions may change rapidly
- duplicate bundles can arrive through multiple paths
- a device may eventually be stolen or physically inspected

## v0.2 storage boundary

The durable queue stores envelope bytes locally and treats them as opaque. The queue itself does not currently provide confidentiality, authenticity or tamper detection for those bytes.

Until the authenticated-envelope layer exists, callers must not treat the v0.2 store as secure storage for plaintext. The JSON store is a research persistence layer, not an anti-forensic or hardened database.

The queue is bounded and uses retry backoff to avoid unbounded growth and tight retry loops. Those controls improve robustness but are not a complete denial-of-service defense.

## Do not claim

Sombra does not claim to hide the existence of radio activity, defeat arbitrary jamming, provide guaranteed anonymity or protect plaintext displayed on a compromised endpoint.

Traffic timing, physical proximity, radio fingerprints and device compromise remain outside what application-layer encryption alone can solve.

## Endpoint loss

The intended endpoint design minimizes retained material: short-lived session state, bounded message history, hardware-backed private keys where available, and explicit device revocation. Hardware-backed identity and authenticated envelope handling remain roadmap requirements.

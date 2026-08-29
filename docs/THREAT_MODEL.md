# Threat model

Sombra is designed for community connectivity during outages, disasters and areas with unreliable infrastructure.

## Protect

- message confidentiality once cryptographic envelopes are implemented
- message authenticity
- integrity of stored bundles
- resilience when individual links or nodes disappear
- unnecessary exposure of social-graph metadata

## Assume

- infrastructure may be unavailable for long periods
- peers may move in and out of contact
- some relays may be lost or untrusted
- radio conditions may change rapidly
- a device may eventually be stolen or physically inspected

## Do not claim

Sombra does not claim to hide the existence of radio activity, defeat arbitrary jamming, provide guaranteed anonymity or protect plaintext displayed on a compromised endpoint.

Traffic timing, physical proximity, radio fingerprints and device compromise remain outside what application-layer encryption alone can solve.

## Endpoint loss

The intended endpoint design minimizes retained material: short-lived session state, bounded message history, hardware-backed private keys where available, and explicit device revocation. These are roadmap requirements, not claims about the current simulator.

# Architecture

Sombra is organized around a transport-independent message core.

```text
Application
    |
Message bundle
    |
Local identity + authenticated envelope
    |
Routing policy
    +-- short-range adapter
    +-- long-range adapter
    +-- delay-tolerant adapter
    +-- intermittent IP adapter
```

## Bundle layer

Bundles carry the minimum routing state needed by the local node: identifier, creation time, expiry, hop budget, priority and payload size. Payload encryption is intentionally outside the simulator so routing experiments cannot accidentally depend on plaintext.

## Routing policy

A node scores links using local observations:

- availability
- delivery probability
- congestion
- energy cost
- latency
- metadata exposure

The score is advisory, local and replaceable. No central server computes routes.

## Delay tolerance

The delay-tolerant layer treats time as part of the network. A bundle can remain stored while no useful peer exists and resume forwarding after a later contact. This is the core mechanism for long outages and sparse topologies.

## Radio adapters

The core does not assume a particular Bluetooth, Wi-Fi or LoRa stack. Hardware adapters should expose measured link properties to the policy engine and remain independently testable.

## Future cryptography

The protocol roadmap includes local identities, authenticated envelopes, forward-secret sessions and hybrid post-quantum key establishment. These will be added only with well-reviewed libraries and explicit interoperability tests.

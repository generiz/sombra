# Architecture

Sombra is organized around a transport-independent message core.

```text
Application
    |
Authenticated envelope        (roadmap)
    |
Message bundle
    |
Deduplication
    |
Durable bundle store
    |
Priority scheduler
    |
Routing policy
    +-- short-range adapter
    +-- long-range adapter
    +-- delay-tolerant adapter
    +-- intermittent IP adapter
```

## Bundle layer

Bundles carry the minimum routing state needed by the local node: identifier, creation time, expiry, hop budget, priority and payload size.

Bundle IDs include local random entropy and are preserved across forwarding hops. The ID is used for deduplication and queue identity; it is not an identity credential or cryptographic signature.

Payload encryption remains outside the routing model so routing experiments cannot accidentally depend on plaintext.

## Deduplication

The in-memory deduplication cache is bounded. Once capacity is reached, the oldest remembered identifier is removed. This limits memory growth while suppressing recently seen bundle loops.

Persistent deduplication across long restarts is not yet implemented.

## Durable store

The v0.2 store persists bundles and opaque envelope bytes locally. Stored entries track:

- bundle metadata
- opaque envelope bytes
- storage time
- failed transmission attempts
- next eligible retry time

The store rejects new entries when its configured capacity is reached. It does not silently discard an existing bundle to admit a new one.

The current JSON-backed store is a research implementation. It is portable and inspectable, not a production database. Appliance-grade storage would require stronger crash consistency, wear considerations, authenticated configuration and migration handling.

## Scheduler

Eligible bundles are ordered locally by priority, retry count and age. Urgent traffic is considered before important and routine traffic. Within a priority class, fewer failed attempts and older bundles are preferred.

Failed transmissions use bounded exponential backoff so a broken path does not create a tight retry loop.

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

Time is part of the network. A bundle can remain stored while no useful peer exists and resume forwarding after a later contact. Expired bundles are removed and hop limits prevent indefinite forwarding.

## Radio adapters

The core does not assume a particular Bluetooth, Wi-Fi or LoRa stack. Hardware adapters should expose measured link properties to the policy engine and remain independently testable.

No live radio adapter is implemented in v0.2.

## Cryptography

The next protocol milestone is a local identity and authenticated-envelope interface. The persistent store already treats envelope bytes as opaque, which keeps that cryptographic layer separate from routing and scheduling.

The roadmap also includes forward-secret sessions and hybrid post-quantum key establishment. These will be added only with reviewed libraries and explicit interoperability test vectors.

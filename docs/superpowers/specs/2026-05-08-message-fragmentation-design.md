# Message Fragmentation

Split and reassemble messages larger than the negotiated `batch_size` across multiple Zenoh frames.

## Motivation

`zenoh-nostd` silently truncates messages exceeding `batch_size`. For drone telemetry, payloads (e.g. sensor arrays, logs) routinely exceed the MTU of embedded TCP/UDP links. Fragmentation is a protocol-level requirement for practical mesh networking.

## Protocol Basis

The Zenoh `FrameHeader` already carries `sn: u32` (sequence number). Protocol convention: consecutive frames with the **same SN** belong to one message. A frame with a different SN (or the end of input) signals message completion.

No new wire types needed. No protocol version bump.

## Design

### TX: Split (`tx.rs`)

In `TransportTx::encode()`, when encoding a `NetworkMessageRef`:

```
if encoded_len <= batch_size - FrameHeader::z_len():
    write as single frame (existing behavior, unchanged)
else:
    let chunk_size = batch_size - FrameHeader::z_len()
    for each chunk in split(payload, chunk_size):
        write FrameHeader with same sn, reliability, qos
        write chunk
```

The `FrameHeader` is always written (existing behavior). For fragmented messages, multiple `FrameHeader`s share the same `sn`. SN only advances when starting a new *logical* message.

### RX: Reassemble (`rx.rs`)

Add a reassembly buffer to `TransportRx`:

```rust
pub struct TransportRx<Buff, const MAX_FRAGMENTS: usize> {
    // ... existing fields ...
    fragment_sn: Option<u32>,
    fragment_buffer: [u8; N],
    fragment_offset: usize,
}
```

`MAX_FRAGMENTS` is a const generic (default: 1, keeping existing behavior for code that doesn't need fragmentation). The buffer size is `batch_size * MAX_FRAGMENTS`.

On receiving a frame:
```
if frame.sn == current_sn || current_sn is None:
    append payload to fragment_buffer
elif frame.sn != current_sn:
    flush fragment_buffer → yield as complete NetworkMessage
    start new fragment with frame.sn
```

A gap in SNs (missing fragment) flushes the partial buffer and starts a new fragment. No retransmission — this is Zenoh, not TCP.

### Configuration

`TransportBuilder` gains `with_max_fragments(usize)` to set `MAX_FRAGMENTS`. `TransportLinkManager` wires it through.

### Error Handling

| Scenario | Behavior |
|----------|----------|
| Single fragment (fits in batch) | Unchanged, zero overhead |
| Missing fragment (SN gap) | Flush partial, start new |
| Quiet message (empty payload) | Inline, no fragmentation |
| Reliability/QoS change mid-fragment | Error (`InvalidAttribute`) |

### Testing

| Test | Level |
|------|-------|
| Single fragment round-trip | Unit (sansio codec) |
| 2-fragment split + reassemble | Unit |
| 3-fragment split + reassemble | Unit |
| SN gap recovery | Unit |
| Mid-fragment QoS change → error | Unit |
| Fragmented TCP integration | Integration |

### Out of Scope

- Fragment reordering (Zenoh assumes reliable transport for ordered delivery)
- Fragment retransmission (Zenoh is not TCP)
- Dynamic buffer sizing (heap-allocated reassembly)

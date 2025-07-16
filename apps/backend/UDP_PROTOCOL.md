# UDP Protocol Documentation for DJ-4LED Frontend

## Overview

The DJ-4LED backend now uses UDP for real-time data transmission instead of WebSocket. This provides lower latency and better performance for LED visualization data streaming.

**Server Address**: `udp://localhost:8081`

## Protocol Structure

### Packet Format

Every UDP packet follows this structure:

```
┌─────────────┬───────┬──────────┬─────────────┬────────────────┬──────────────┬─────────┐
│ Type (1B)   │ Flags │ Sequence │ Fragment ID │ Fragment Count │ Payload Size │ Payload │
│             │ (1B)  │ (4B)     │ (2B)        │ (2B)           │ (2B)         │ (var)   │
└─────────────┴───────┴──────────┴─────────────┴────────────────┴──────────────┴─────────┘
```

Total header size: 12 bytes

### Packet Types

| Type                | Value | Description                |
| ------------------- | ----- | -------------------------- |
| Connect             | 0x01  | Initial connection request |
| Disconnect          | 0x02  | Clean disconnection        |
| Ping                | 0x03  | Keep-alive ping            |
| Pong                | 0x04  | Ping response              |
| Ack                 | 0x05  | Acknowledgment             |
| Command             | 0x10  | Command from client        |
| FrameData           | 0x20  | LED frame data             |
| FrameDataCompressed | 0x21  | Compressed LED frame data  |
| SpectrumData        | 0x30  | Audio spectrum data        |

### Flags

Flags are bitwise combinable:

| Flag          | Value | Description                            |
| ------------- | ----- | -------------------------------------- |
| COMPRESSED    | 0x01  | Payload is compressed with gzip        |
| FRAGMENTED    | 0x02  | Packet is part of a fragmented message |
| LAST_FRAGMENT | 0x04  | This is the last fragment              |
| REQUIRES_ACK  | 0x08  | Sender expects acknowledgment          |

## Connection Flow

### 1. Establishing Connection

Send a Connect packet to the server:

```rust
// Packet structure
{
    packet_type: 0x01,  // Connect
    flags: 0x00,        // Or 0x01 if compression is supported
    sequence: 0,
    fragment_id: 0,
    fragment_count: 1,
    payload: []         // Empty
}
```

The server will respond with an Ack packet if successful.

### 2. Receiving Data

The server continuously sends two types of data:

#### Frame Data (LED visualization)

-   Type: 0x20 (uncompressed) or 0x21 (compressed)
-   Payload structure:
    ```
    ┌────────┬────────┬────────┬──────────┐
    │ Width  │ Height │ Format │ RGB Data │
    │ (2B)   │ (2B)   │ (1B)   │ (var)    │
    └────────┴────────┴────────┴──────────┘
    ```
-   Default resolution: 64x64 pixels
-   Format: 0x01 = RGB (3 bytes per pixel)
-   Data order: Row-major, top-to-bottom, left-to-right

#### Spectrum Data (Audio frequencies)

-   Type: 0x30
-   Payload structure:
    ```
    ┌─────────────┬───────────────────┐
    │ Band Count  │ Float Values      │
    │ (2B)        │ (4B × band_count) │
    └─────────────┴───────────────────┘
    ```
-   Typically 32 frequency bands
-   Values are normalized (0.0 to 1.0)

### 3. Sending Commands

Commands are sent with packet type 0x10:

#### Set Effect Command

```rust
// Command ID: 0x01
// Payload: [0x01, effect_id(4B)]
{
    packet_type: 0x10,
    payload: [0x01, 0x05, 0x00, 0x00, 0x00]  // Set effect ID 5
}
```

#### Set Color Mode Command

```rust
// Command ID: 0x02
// Payload: [0x02, mode_string_bytes...]
{
    packet_type: 0x10,
    payload: [0x02, "rainbow".as_bytes()...]
}
```

#### Set Custom Color Command

```rust
// Command ID: 0x03
// Payload: [0x03, r(4B), g(4B), b(4B)]
{
    packet_type: 0x10,
    payload: [0x03, r_float_bytes, g_float_bytes, b_float_bytes]
}
```

#### Set Parameter Command

```rust
// Command ID: 0x04
// Payload: [0x04, name_len(2B), name_bytes, value_len(2B), value_bytes]
{
    packet_type: 0x10,
    payload: [0x04, name_len, name_bytes..., value_len, value_bytes...]
}
```

### 4. Keep-Alive

Send a Ping packet every 30 seconds to maintain the connection:

```rust
{
    packet_type: 0x03,  // Ping
    sequence: current_sequence++,
    payload: []
}
```

## Fragmentation

Large messages (> 1460 bytes) are automatically fragmented:

1. Each fragment has the FRAGMENTED flag set
2. `fragment_id` starts at 0 and increments
3. `fragment_count` indicates total fragments
4. Last fragment has LAST_FRAGMENT flag
5. Reassemble by concatenating payloads in order

## Compression

When receiving compressed data (COMPRESSED flag set):

1. The payload is gzip-compressed
2. Decompress before processing
3. Compressed frames use packet type 0x21

## Performance Optimizations

1. **MTU Consideration**: Maximum packet size is 1472 bytes (typical MTU minus IP/UDP headers)
2. **Frame Rate**: Server sends at ~40 FPS for smooth visualization
3. **Downscaling**: Frames are downscaled to 64x64 to reduce bandwidth
4. **Spectrum Reduction**: Audio spectrum reduced to 32 bands
5. **Delta Updates**: Data only sent when changed (with 1Hz minimum update)

## Error Handling

1. **Packet Loss**: UDP doesn't guarantee delivery. Implement frame interpolation on client side
2. **Out-of-Order**: Check sequence numbers and reorder if necessary
3. **Timeout**: Remove server from active list if no data received for 60 seconds

## Example Rust Client Implementation

```rust
use std::net::UdpSocket;

// Create socket
let socket = UdpSocket::bind("0.0.0.0:0")?;
socket.connect("localhost:8081")?;

// Send connect packet
let connect_packet = create_connect_packet();
socket.send(&connect_packet)?;

// Receive loop
let mut buf = [0u8; 2048];
loop {
    match socket.recv(&mut buf) {
        Ok(len) => {
            let packet = parse_packet(&buf[..len])?;
            match packet.packet_type {
                0x20 | 0x21 => handle_frame_data(packet),
                0x30 => handle_spectrum_data(packet),
                _ => {}
            }
        }
        Err(e) => {
            // Handle error
        }
    }
}
```

## Binary Format Examples

### Connect Packet (12 bytes)

```
01 00 00 00 00 00 00 00 01 00 00 00
│  │  └─────┴─────┘ └──┴──┘ └──┴──┘
│  │     Sequence    Frag    Payload
│  └─ Flags (0x00 = no compression)
└─ Type (Connect)
```

### Frame Data Packet (compressed, 64x64 RGB)

```
21 01 2A 00 00 00 00 00 01 00 [size] [gzip data...]
│  │  └─────┴─────┘ └──┴──┘    └──┴──┘
│  │     Sequence    Frag     Payload size
│  └─ Flags (0x01 = compressed)
└─ Type (FrameDataCompressed)
```

### Command Packet (Set Effect ID 3)

```
10 00 15 00 00 00 00 00 01 00 05 00 01 03 00 00 00
│  │  └─────┴─────┘            └──┴──┘ │  └─────┴─────┘
│  │     Sequence              Payload  │   Effect ID (3)
│  └─ Flags                    size     └─ Command ID
└─ Type (Command)
```

## Migration from WebSocket

Key differences when migrating from WebSocket:

1. **Connection**: UDP is connectionless - send Connect packet to register
2. **Reliability**: Implement client-side buffering and interpolation
3. **Message Format**: Binary protocol instead of JSON
4. **Fragmentation**: Handle manually for large messages
5. **Keep-Alive**: Send periodic pings to maintain registration

## Notes for Tauri Frontend

Since you're using Tauri with Rust:

1. Use `std::net::UdpSocket` or `tokio::net::UdpSocket` for async
2. Consider using `bincode` or manual byte manipulation for packet serialization
3. Implement a packet reassembly buffer for fragmented messages
4. Use `flate2` crate for gzip decompression
5. Consider running UDP receiver in a separate thread/task to avoid blocking UI

## Testing

Test the UDP server with netcat:

```bash
# Send a hex-encoded connect packet
echo -n -e "\x01\x00\x00\x00\x00\x00\x00\x00\x01\x00\x00\x00" | nc -u localhost 8081
```

Or use the included test client (if provided).

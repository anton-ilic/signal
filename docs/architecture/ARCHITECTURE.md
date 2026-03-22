# SIGNAL Architecture Plan

## Overview

**SIGNAL** is a wireless call-button system that lets a user press a physical button to request assistance.

The system has four main parts:

- **Button**: battery-powered device that sends a local wireless signal
- **Receiver**: plugged-in device that listens for the button, plays a chime, and forwards events
- **Backend**: Rust server that stores events, manages devices, and sends notifications
- **App**: mobile app for receiving alerts and viewing event history

---

## High-Level Flow

[ Button ] --(local radio)--> [ Receiver ] --(HTTPS/Wi-Fi)--> [ Backend ] --(push)--> [ App ]
                                      |
                                      +--> [ Local Chime / Speaker ]

---

## Repo Structure

SIGNAL/
  app/
  backend/
  button/
  receiver/
  .gitignore
  LICENSE
  README.md
  ARCHITECTURE.md

---

## Component Breakdown

## 1. Button (/button)

### Purpose

The button is the battery-powered device the user physically presses to request help.

### Recommended Stack

- Hardware family: Nordic nRF52
- Language: C or C++
- Radio: BLE advertising or custom 2.4 GHz packets
- Power source: coin cell or small battery

### Responsibilities

- sleep most of the time
- wake when the button is pressed
- increment an event counter
- send a small radio packet
- optionally retry transmission
- return to sleep

### Button Packet (Conceptual)

```json
{
  "button_id": "uuid-or-device-id",
  "event_counter": 123,
  "signature": "auth-value"
}
```

Note: In practice, this will be a compact binary packet.

### Notes

- must be low power
- no Wi-Fi or internet
- minimal logic only

---

## 2. Receiver (/receiver)

### Purpose

The receiver listens for button signals and bridges them to the backend.

### Recommended Stack

Option A (embedded):
- ESP32
- C/C++

Option B (Linux-based):
- Raspberry Pi Zero 2 W
- Rust or C++

### Responsibilities

- listen for radio packets
- validate packets
- prevent duplicate/replay events
- trigger local chime immediately
- send event to backend
- maintain heartbeat

### Event Flow

1. receive packet
2. validate
3. play chime
4. send to backend

### Receiver -> Backend Example

`POST /v1/events/button-press`

```json
{
  "button_id": "button-123",
  "event_counter": 123,
  "received_at": "timestamp"
}
```

### Notes

- must work offline (local chime still works)
- queue events if backend unavailable

---

## 3. Backend (/backend)

### Purpose

The backend processes events, manages devices, and sends notifications.

### Recommended Stack

- Rust
- Axum
- Tokio
- PostgreSQL
- SQLx
- Serde
- Tracing

### Responsibilities

- authenticate receivers
- ingest button events
- store event history
- prevent replay attacks
- send push notifications
- provide app API

---

## 4. App (/app)

### Purpose

User-facing interface.

### Recommended Stack

- Flutter (cross-platform)

### Responsibilities

- receive push notifications
- display alerts/history
- manage devices
- handle pairing/setup

---

## Backend Architecture

### Folder Structure

backend/
  src/
    main.rs
    config.rs
    error.rs
    routes/
    domain/
    db/
    services/
    middleware/
  migrations/
  tests/
  Cargo.toml
  Dockerfile
  docker-compose.yml

---

## Core Backend Modules

### Auth
- receiver authentication
- user authentication

### Events
- ingest button presses
- deduplicate using event_counter

### Devices
- manage buttons and receivers

### Notifications
- send push via APNs / FCM

---

## Data Model

users
- id
- email
- created_at

receivers
- id
- user_id
- name
- auth_token
- last_seen_at

buttons
- id
- user_id
- receiver_id
- label

button_events
- id
- button_id
- receiver_id
- event_counter
- pressed_at

push_tokens
- id
- user_id
- platform
- token

---

## Event Flow (Detailed)

1. user presses button
2. button transmits packet
3. receiver receives and validates
4. receiver plays chime
5. receiver sends event to backend
6. backend validates and stores event
7. backend sends push notification
8. app receives notification

---

## Communication Layers

Button -> Receiver:
- BLE or custom 2.4 GHz
- small binary packet

Receiver -> Backend:
- HTTPS REST
- JSON

Backend -> App:
- push notifications

App -> Backend:
- HTTPS REST

---

## Reliability

- local chime works without internet
- receiver can queue events
- button may retry transmission
- backend deduplicates events

---

## Security

- button packets include ID + counter + signature
- receiver authenticated with token
- HTTPS required
- replay protection via event_counter

---

## Development Phases

Phase 1:
- backend event ingestion
- DB storage
- simulated receiver

Phase 2:
- real receiver
- push notifications
- app basic UI

Phase 3:
- real button
- pairing flow
- device management

---

## Future Extensions

- multiple receivers
- configurable sounds
- OTA updates
- analytics dashboard

---

## Summary

SIGNAL uses a hybrid design:

- Local: button -> receiver -> instant chime
- Cloud: receiver -> backend -> app notification

This provides low latency, reliability, and scalability.

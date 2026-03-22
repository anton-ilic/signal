# SIGNAL

A wireless call-button system that connects physical buttons to mobile apps through a cloud backend.

## How It Works

The system creates a flow from hardware to the cloud:

`button → receiver → backend → app`

- **Button**: Firmware for the wireless call button
- **Receiver**: Wireless receiver that catches button signals
- **Backend**: Cloud service that ingests events, authenticates receivers, and manages devices
- **App**: React Native mobile app for users to monitor and respond

## Components

- [App](app/README.md) — React Native mobile app
- [Backend](backend/README.md) — Rust cloud backend
- [Button](button/README.md) — Button firmware (C/C++)
- [Receiver](receiver/README.md) — Receiver firmware (C/C++)

For the full architecture, see [docs/architecture/ARCHITECTURE.md](docs/architecture/ARCHITECTURE.md).

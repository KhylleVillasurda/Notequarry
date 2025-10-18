# NoteQuarry

**Version:** 1.0  
**Date:** October 18, 2025  
**Author:** Khylle P. Villasurda

## Executive Summary

**NoteQuarry** is a secure, high-performance desktop note-taking application designed for users who value privacy and efficiency. Supporting multiple content modes—**Book** (paginated) and **Note** (freeform with checkboxes)—it offers a versatile environment for organizing information. The system employs end-to-end encryption and features cloud synchronization to keep your notes private and accessible wherever you go.

### Core Features

- **Multi-Mode Content Creation**
  - **Book Mode:** Structured, paginated notes for organized reading and writing.
  - **Note Mode:** Freeform notes with checkbox support for lists and tasks.
- **Strong Security**
  - End-to-end encryption using ChaCha20-Poly1305.
  - Secure key derivation with Argon2id.
- **Cloud Synchronization**
  - Encrypted sync to Google Drive.
  - Proton Drive integration coming soon.
- **Performance-First**
  - Handles 10,000+ entries with instant full-text search.
  - Optimized for speed and responsiveness.

## System Architecture

### Technology Stack

- **Frontend:** [Slint 1.8](https://slint.dev/) (Rust-based declarative UI framework)
- **Backend:** Rust (implements core logic, encryption, cloud sync)
- **Database:** SQLite with FTS5 (Full-text search for fast, flexible querying)
- **Encryption:** ChaCha20-Poly1305 for note data, Argon2id for key derivation
- **Cloud APIs:** 
  - Google Drive API (current)
  - Proton Drive API (upcoming)

---

## Getting Started

1. **Build Requirements**
   - Rust (latest stable)
   - [Slint](https://slint.dev/) 1.8
   - SQLite
2. **Setup**
   - Clone the repository
   - Follow build instructions in the repository for your OS
   - Configure cloud sync (Google Drive) in settings

## Security Notes

- All notes are encrypted locally before sync.
- Only you hold the encryption keys (derived securely via Argon2id).
- Zero-knowledge cloud sync: no plaintext data is ever uploaded.

## Roadmap

- [x] Google Drive sync
- [x] Proton Drive sync
- [ ] Mobile client (future)

## License

See [LICENSE](LICENSE) for details.

---

**NoteQuarry** is built for privacy, speed, and flexibility, empowering you to organize your thoughts securely and efficiently.

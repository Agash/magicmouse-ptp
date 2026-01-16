# MagicMouse-PTP-Rust 🖱️🦀

A modern, memory-safe **Windows 11 Driver** for the **Apple Magic Mouse** (Bluetooth).

This project implements a **Hybrid HID Minidriver** using the **Windows Driver Framework (WDF)** and **Rust**. Unlike generic drivers, this project maps the Magic Mouse touch surface to the **Windows Precision Touchpad (PTP)** protocol, enabling native Windows 11 gestures, fluid scrolling, and multi-touch support.

## ✨ Features

- **Hybrid Input:** Simultaneously acts as a high-resolution Mouse and a Precision Touchpad.
- **Native Gestures:** Support for Windows 11 multi-finger swipes, pinch-to-zoom, and virtual desktops.
- **Precision Scrolling:** Pixel-perfect "Mac-style" smooth scrolling (no more notched scrolling).
- **Memory Safety:** Built entirely in Rust to eliminate common kernel-level vulnerabilities (BSODs).
- **Low Latency:** Optimized Bluetooth HID report parsing based on the Linux `hid-magicmouse` implementation.

## 🛠️ Technical Architecture

Based on the `windows-drivers-rs` framework, the driver intercepts raw Apple HID Report ID `0x10` packets and translates them into:

1. **Top-Level Collection 0:** Standard HID Mouse (X/Y Relative Movement).
2. **Top-Level Collection 1:** Windows PTP (Absolute Multi-touch data).

## 🚀 Getting Started

### Prerequisites

- Windows 11 + WDK (Windows Driver Kit) 2026
- Rust Nightly + `cargo-wdk`
- LLVM/Clang

### Installation

1. Enable **Test Signing** mode: `bcdedit /set testsigning on`
2. Build the driver: `cargo wdk build`
3. Install via the provided `.inf` (See `/dist` for instructions).

## 📜 License

**CC-BY-NC-4.0** (Creative Commons Attribution-NonCommercial 4.0 International)

- **You are free to:** Share and Adapt the code.
- **You must:** Give appropriate credit.
- **You may NOT:** Use this material for **commercial purposes**.

If you wish to use this driver in a commercial product or environment, please contact the author for a commercial license.

---

_Disclaimer: This project is not affiliated with or endorsed by Apple Inc._

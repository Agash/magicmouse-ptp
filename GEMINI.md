# Gemini Code Assist Context: MagicMouse-PTP-Rust

## 🎯 Project Goal
Developing a **Hybrid HID Minidriver** for the Apple Magic Mouse (Bluetooth) on Windows 11.
The driver must expose two Top-Level Collections (TLCs):
1. **Mouse TLC (ID 0x01)**: Standard relative movement and buttons.
2. **PTP TLC (ID 0x02)**: Windows Precision Touchpad for native gestures.

## 🛠️ Tech Stack
- **Language**: Rust (`no_std`)
- **Framework**: Windows Driver Framework (KMDF) via `windows-drivers-rs`
- **Reference**: Linux kernel `drivers/hid/hid-magicmouse.c` for protocol mapping.

## 🚦 Implementation Status
- [x] WDF Driver Boilerplate & `DriverEntry`.
- [x] Hybrid HID Descriptor (Initial draft in `descriptors.rs`).
- [x] Magic Sequence (MM1/MM2) on D0Entry.
- [x] Continuous Reader for Report ID `0x10`.
- [x] Simulated Mouse Movement (Timer-based).
- [ ] **NEXT: Multi-touch (PTP) Report Parsing**.
- [ ] **NEXT: PTP Data Translation Layer**.

## Current Debugging Task
- Resolving 0xC000009C in EvtDeviceD0Entry.
- Goal: Make Magic Sequence non-blocking and ensure HID IOCTLs are handled correctly to satisfy mshidkmdf.

## ⚠️ Critical Rules for Code Generation
1. **Memory Safety**: Use `#[repr(C, packed)]` for all structs mapping to hardware reports. Avoid `unsafe` except for `wdk-sys` FFI calls.
2. **No Std**: Use `core::` and `wdk_alloc`. Never use `std::`.
3. **PTP Protocol**: Follow Windows Precision Touchpad standards. Finger data must include `ContactID`, `TipSwitch`, `X`, and `Y`.
4. **Linux Reference**: When generating parsing logic, refer to the byte offsets found in the Linux `hid-magicmouse` driver for Report ID `0x10`.

## 📂 Key Files
- `driver/src/lib.rs`: Main WDF logic, IOCTL handling, and Continuous Reader.
- `driver/src/descriptors.rs`: HID Report Descriptors and Report Structs.
- `driver/magic_mouse.inx`: Installation configuration.
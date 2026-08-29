# ✦ LarusX

> **Engineered by Syojhin & Lara**  
> *Zero-Overhead Competitive Display Tuner, Black Equalizer & Subpixel Crosshair Suite for Windows.*

[![License: MIT](https://img.shields.io/badge/License-MIT-emerald.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%2010%20%2F%2011-blue.svg)](https://microsoft.com/windows)
[![Vendor](https://img.shields.io/badge/Hardware-NVIDIA%20%7C%20AMD%20%7C%20Intel-purple.svg)](#hardware-compatibility)
[![Laptops](https://img.shields.io/badge/Laptops-NVIDIA%20Optimus%20%26%20Hybrid%20Ready-orange.svg)](#laptop--nvidia-optimus-support)
[![Performance](https://img.shields.io/badge/FPS%20Impact-0.0%25-brightgreen.svg)](#architecture)
[![Anti--Cheat](https://img.shields.io/badge/Anti--Cheat-100%25%20Safe%20%28Zero%20Hook%29-success.svg)](#anti-cheat-safety)

---

## ⚡ Overview

**LarusX** is a high-performance, lightweight display tuning and reticle HUD utility built in pure Rust. It transforms ordinary 1080p/1440p gaming panels into tournament-grade competitive displays by providing:

- **0.0% FPS Overhead:** Directly configures Desktop Window Manager (DWM) hardware composition matrices. Zero video hooks, zero screen grabbing, zero OBS capture lag.
- **Laptop & NVIDIA Optimus Ready:** Bypasses the classic hybrid graphics lock that prevents NVIDIA Control Panel from adjusting vibrance on laptop screens.
- **Hardware-Level Black Equalizer:** Recovers hidden enemy geometry in deep shadows without blowing out skies or highlights.
- **Subpixel Anti-Aliased Crosshair HUD:** 32-bit ARGB overlay with subpixel precision that ensures reticles are dead-center on physical monitor rasters.
- **Auto-Process Detection:** Dynamically switches color profiles when games are focused and resets upon Alt-Tabbing.
- **GPU-Vendor Agnostic:** Full native compatibility with **NVIDIA GeForce**, **AMD Radeon**, and **Intel Arc** GPUs.

---

## 💻 Laptop & NVIDIA Optimus Support

Traditional tools (such as VibranceGUI) rely on proprietary NVIDIA driver calls (`NvAPI_SetDVCLevel`). On gaming laptops with **NVIDIA Optimus / Hybrid Graphics**, the built-in screen is physically wired to the Intel or AMD integrated GPU (iGPU), causing NVIDIA Control Panel and NVAPI tools to **fail completely**.

**LarusX completely eliminates this limitation:**
- Intercepts composition at the **Windows DWM Compositor** layer.
- Works seamlessly on **any gaming laptop screen**, whether running through an Intel iGPU, AMD Radeon 780M/680M APU, or discrete MUX switch.
- Also supports external monitors connected via HDMI, DisplayPort, or USB-C Thunderbolt.

---

## 🎮 Hardware Compatibility

LarusX avoids proprietary driver hacks in favor of standardized Windows Display Driver Model (WDDM) APIs:

| GPU Architecture | Status | Supported Features |
| :--- | :--- | :--- |
| **NVIDIA GeForce** (RTX 50/40/30/20, GTX 16/10) |  Full Native | 5x5 Color Matrix, GDI Gamma LUT, Subpixel Crosshair |
| **AMD Radeon** (RX 7000/6000/5000, Vega, Polaris) |  Full Native | 5x5 Color Matrix, GDI Gamma LUT, Subpixel Crosshair |
| **Intel Arc / Iris Xe** (A770, A750, B580, Integrated) |  Full Native | 5x5 Color Matrix, GDI Gamma LUT, Subpixel Crosshair |
| **Gaming Laptops (Optimus / Hybrid MUX)** |  Full Native | 5x5 Color Matrix, GDI Gamma LUT, Subpixel Crosshair |

---

## 🛡️ Anti-Cheat Safety

LarusX is **100% safe** for use in competitive shooters protected by kernel-level and heuristic anti-cheats, including:
- **Riot Vanguard** (*Valorant*)
- **Valve Anti-Cheat / VAC Live** (*CS2*)
- **Easy Anti-Cheat** (*Marvel Rivals, Apex Legends*)
- **BattlEye** (*Rainbow Six Siege, Destiny 2*)
- **Blizzard Defense Matrix** (*Overwatch 2*)

### Why is it safe?
1. **Zero Process Memory Access:** Does not open process handles with `PROCESS_VM_READ` or `PROCESS_VM_WRITE`.
2. **Zero DLL Injection / Hooking:** Does not inject `.dll` files or hook DirectX/Vulkan render loops (`Present()`, `EndScene()`).
3. **Native Desktop Window Composition:** Uses Microsoft's documented `Magnification.dll` (`MagSetFullscreenColorEffect`) and Win32 `SetDeviceGammaRamp`.

---

## 🎯 Features

### 1. 5x5 DWM Color Transformation Matrix
- **Digital Vibrance & Saturation:** Fine-tuned 0% to 300% saturation scaling.
- **Daltonisation Modes:** Algorithmic outline separation filters (Deuteranopia, Protanopia, Tritanopia, and PvP High-Vis).
- **Independent RGB Gain Tuning:** Direct multiplier adjustments for Red, Green, and Blue channels to neutralize map plaster tints and isolate enemy outlines.

### 2. Black Equalizer & Gamma Transfer Function
- **Selective Shadow Lift:** Non-linear cubic curve lifts luminance exclusively in the `0.0 – 0.4` dark shadow region while tapering off cleanly before mid-tones and highlights.
- **Hardware Gamma Ramp:** Smooth 256-step GDI LUT prevents color banding and artifacting.

### 3. 32-Bit ARGB Subpixel Crosshair HUD
- **Smooth Anti-Aliasing:** High-DPI GDI+ rendering with `SmoothingModeAntiAlias` and `PixelOffsetModeHighQuality`.
- **True Optical Center:** Eliminates 0.5px / 1px Unreal Engine 4 integer UI quantization rounding errors.
- **Pro Presets:**
  - `🎯 TenZ Micro-Dot` (Cyan subpixel dot + dark border)
  - `□ Demon1 Box` (Hollow micro-box headshot frame)
  - `+ ScreaM One-Tap` (Crisp tournament crosshair)
  - `○ Radiant Ring` (Smooth circle sight + center dot)
  - `^ Chevron Apex` (Inverted headhunter caret)

### 4. Direct Click-to-Type Precision
- Click directly on any value box in the UI and type exact numbers (e.g. `135%`, `110%`, `0.95`, `1.20`) with instant Enter confirmation.

---

## 🚀 Hotkeys

| Hotkey | Action |
| :--- | :--- |
| **`F7`** | Global Master Bypass Toggle (Enable / Disable Color Matrix & Gamma) |
| **`F8`** | Toggle Crosshair HUD Overlay On / Off |

---

## 📦 Building from Source

### Prerequisites
- **Rust Toolchain:** Stable channel (`rustup default stable`)
- **OS:** Windows 10 / 11 (64-bit)

### Build Commands

```bash
# Clone the repository
git clone https://github.com/your-username/larusx.git
cd larusx

# Build optimized release binary
cargo build --release
```

The compiled standalone executable will be located at `target/release/larusx.exe` (approx. 3.7 MB, zero external runtime dependencies).

---

## 📄 License

Distributed under the **MIT License**. See [`LICENSE`](LICENSE) for more information.

---

<div align="center">
  <sub>✦ Crafted with precision by <b>Syojhin</b> & <b>Lara</b></sub>
</div>

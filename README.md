# Clock

This [Pico](https://www.raspberrypi.com/documentation/microcontrollers/raspberry-pi-pico.html)-based clock uses a dedicated oscillator for the built-in RP2040 [RTC](https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf#section_rtc), displays time using seven-segment indicators, features capacitive touch-sensing buttons, and a custom built-in NiMH charging circuit.

The estimated run-time on a single charge is _one week_ (yes, _just_ one week).

<p width="100%" align="justify">
<img alt="PCB 3D view, top" width="49.5%" src="images/3D%20top.png">
<img alt="PCB 3D view, bottom" width="49.5%" src="images/3D%20bottom.png">
<img alt="Photo, combined" width="100%" src="images/Photo%20comb.jpg">
</p>

When assembling, refer to the [interactive BOM](https://htmlpreview.github.io/?https://github.com/werediver/clock/blob/main/KiCad/bom/ibom.html) page.

## NiMH battery charger

```mermaid
---
title: Charger state machine
---
stateDiagram-v2
    [*] --> Hold
    Hold --> Charge: ext_power && v < HIGH
    Charge --> Charged: d(v) ≤ NDV || is_adc_saturated() || is_timed_out()
    Charged --> Charge: v < HIGH
    Charge --> Hold: !ext_power
    Charged --> Hold: !ext_power
```

- [ ] Add a time-out for `v ≥ NiMH_HIGH` (1–2h?)

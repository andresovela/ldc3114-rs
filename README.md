# LDC3114

Driver crate for the TI [LDC3114] Inductance-to-Digital Converter.
Compatible with [embedded-hal] and [embedded-hal-async] traits.

## Example usage

```rust
let mut inductance_sensor = Ldc3114::new(inductance_sensor_i2c);

// Set the device in configuration mode
inductance_sensor.config_mode().await.unwrap();

// Wait until the registers are ready to write
loop {
    if inductance_sensor.is_ready_to_write().await.unwrap() {
        break;
    }
    Timer::after_millis(5).await;
}

// Your setup
inductance_sensor.set_normal_scan_rate(ScanRate::Lowest).await.unwrap();
inductance_sensor.set_low_power_scan_rate(LowPowerScanRate::Low).await.unwrap();
inductance_sensor.enable_button_press_detection_algorithm(false).await.unwrap();

// Set the device in normal mode
inductance_sensor.normal_mode().await.unwrap();

// Wait until the chip is ready
loop {
    if inductance_sensor.is_chip_ready().await.unwrap() {
        break;
    }
    Timer::after_millis(5).await;
}

loop {
    // Read status to update raw data registers
    let _ = inductance_sensor.read_status().await.unwrap();
    let ch0 = inductance_sensor.read_raw_data(Channel0).await.unwrap();
    let ch1 = inductance_sensor.read_raw_data(Channel1).await.unwrap();
    let ch2 = inductance_sensor.read_raw_data(Channel2).await.unwrap();
    let ch3 = inductance_sensor.read_raw_data(Channel3).await.unwrap();
}
```

## Alternative setup

If you have many configurations to set, set up a const `DeviceConfig`:
```rust
const LDC3114_CONFIG: DeviceConfig = DeviceConfig {
    scan_rate: ScanRate::Highest,
    ..DeviceConfig::const_default()
};
```

and then call

```rust
inductance_sensor.set_device_configuration(&LDC3114_CONFIG).await.unwrap();
```

## Resources

- [Datasheet]
- [Application Note]

## License

This work is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

[LDC3114]: https://www.ti.com/product/LDC3114
[embedded-hal]: https://docs.rs/embedded-hal/latest/embedded_hal/
[embedded-hal-async]: https://docs.rs/embedded-hal-async/latest/embedded_hal_async/
[Datasheet]: https://www.ti.com/lit/gpn/ldc3114
[Application Note]: https://www.ti.com/lit/pdf/snoaa76

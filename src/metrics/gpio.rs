//! GPIO (General Purpose Input/Output) monitoring for Raspberry Pi.
//!
//! This module provides functionality to read GPIO pin states and configuration.
//! It's feature-gated to allow compilation on non-Raspberry Pi systems.

use crate::error::{Result, SystemError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// GPIO pin status and configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpioStatus {
    /// Available GPIO pins on the system
    pub available_pins: Vec<u8>,
    /// Current pin states (pin_number -> state)
    pub pin_states: HashMap<u8, PinState>,
    /// Pin functions (pin_number -> function)
    pub pin_functions: HashMap<u8, PinFunction>,
    /// Whether GPIO access is available
    pub gpio_available: bool,
}

/// State of a GPIO pin.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PinState {
    /// Pin is configured as output and driven low
    Low,
    /// Pin is configured as output and driven high  
    High,
    /// Pin is configured as input
    Input,
    /// Pin state is unknown or inaccessible
    Unknown,
}

/// Function/mode of a GPIO pin.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PinFunction {
    /// Pin is configured as input
    Input,
    /// Pin is configured as output
    Output,
    /// Pin is used for PWM (Pulse Width Modulation)
    Pwm,
    /// Pin is used for SPI communication
    Spi,
    /// Pin is used for I2C communication
    I2c,
    /// Pin is used for UART communication
    Uart,
    /// Pin has an alternative function
    Alt(u8),
    /// Pin function is unknown
    Unknown,
}

/// Trait for GPIO operations.
pub trait GpioProvider {
    /// Read the current state of all GPIO pins.
    fn read_gpio_status(&mut self) -> Result<GpioStatus>;

    /// Check if a specific pin is available.
    fn is_pin_available(&self, pin: u8) -> bool;

    /// Read the state of a specific pin.
    fn read_pin(&mut self, pin: u8) -> Result<PinState>;
}

#[cfg(feature = "gpio")]
mod raspberry_pi {
    use super::*;
    use rppal::gpio::{Gpio, Mode};
    use std::sync::Arc;

    /// Raspberry Pi GPIO provider using rppal.
    pub struct RaspberryPiGpio {
        gpio: Arc<Gpio>,
        // Cache available pins to avoid repeated system calls
        available_pins: Vec<u8>,
    }

    impl RaspberryPiGpio {
        /// Create a new Raspberry Pi GPIO provider.
        pub fn new() -> Result<Self> {
            let gpio = Gpio::new().map_err(|e| {
                SystemError::gpio_error(format!("Failed to initialize GPIO: {}", e))
            })?;

            // Standard Raspberry Pi GPIO pins (0-27 are typically available)
            let available_pins: Vec<u8> = (0..=27).collect();

            Ok(Self {
                gpio: Arc::new(gpio),
                available_pins,
            })
        }
    }

    impl GpioProvider for RaspberryPiGpio {
        fn read_gpio_status(&mut self) -> Result<GpioStatus> {
            let mut pin_states = HashMap::new();
            let mut pin_functions = HashMap::new();

            for &pin_num in &self.available_pins {
                // Attempt to get pin info without claiming the pin
                match self.gpio.get(pin_num) {
                    Ok(pin) => {
                        let function = match pin.mode() {
                            Mode::Input => PinFunction::Input,
                            Mode::Output => PinFunction::Output,
                            Mode::Pwm => PinFunction::Pwm,
                            Mode::Spi => PinFunction::Spi,
                            Mode::I2c => PinFunction::I2c,
                            Mode::Uart => PinFunction::Uart,
                            Mode::Alt(alt) => PinFunction::Alt(alt),
                        };

                        let state = match pin.mode() {
                            Mode::Input => PinState::Input,
                            Mode::Output => {
                                if pin.is_set_high() {
                                    PinState::High
                                } else {
                                    PinState::Low
                                }
                            }
                            _ => PinState::Unknown,
                        };

                        pin_states.insert(pin_num, state);
                        pin_functions.insert(pin_num, function);
                    }
                    Err(_) => {
                        // Pin might be in use or inaccessible
                        pin_states.insert(pin_num, PinState::Unknown);
                        pin_functions.insert(pin_num, PinFunction::Unknown);
                    }
                }
            }

            Ok(GpioStatus {
                available_pins: self.available_pins.clone(),
                pin_states,
                pin_functions,
                gpio_available: true,
            })
        }

        fn is_pin_available(&self, pin: u8) -> bool {
            self.available_pins.contains(&pin)
        }

        fn read_pin(&mut self, pin: u8) -> Result<PinState> {
            if !self.is_pin_available(pin) {
                return Err(SystemError::gpio_error(format!(
                    "Pin {} is not available",
                    pin
                )));
            }

            let gpio_pin = self.gpio.get(pin).map_err(|e| {
                SystemError::gpio_error(format!("Failed to access pin {}: {}", pin, e))
            })?;

            let state = match gpio_pin.mode() {
                Mode::Input => PinState::Input,
                Mode::Output => {
                    if gpio_pin.is_set_high() {
                        PinState::High
                    } else {
                        PinState::Low
                    }
                }
                _ => PinState::Unknown,
            };

            Ok(state)
        }
    }
}

#[cfg(not(feature = "gpio"))]
mod mock {
    use super::*;

    /// Mock GPIO provider for systems without GPIO support.
    pub struct MockGpio;

    impl MockGpio {
        pub fn new() -> Result<Self> {
            Ok(Self)
        }
    }

    impl GpioProvider for MockGpio {
        fn read_gpio_status(&mut self) -> Result<GpioStatus> {
            Ok(GpioStatus {
                available_pins: Vec::new(),
                pin_states: HashMap::new(),
                pin_functions: HashMap::new(),
                gpio_available: false,
            })
        }

        fn is_pin_available(&self, _pin: u8) -> bool {
            false
        }

        fn read_pin(&mut self, pin: u8) -> Result<PinState> {
            Err(SystemError::gpio_error(format!(
                "GPIO not available on this system (attempted to read pin {})",
                pin
            )))
        }
    }
}

// Re-export the appropriate GPIO provider
#[cfg(feature = "gpio")]
pub use raspberry_pi::RaspberryPiGpio as DefaultGpioProvider;

#[cfg(not(feature = "gpio"))]
pub use mock::MockGpio as DefaultGpioProvider;

impl Default for GpioStatus {
    fn default() -> Self {
        Self {
            available_pins: Vec::new(),
            pin_states: HashMap::new(),
            pin_functions: HashMap::new(),
            gpio_available: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpio_status_default() {
        let status = GpioStatus::default();
        assert!(!status.gpio_available);
        assert!(status.available_pins.is_empty());
        assert!(status.pin_states.is_empty());
    }

    #[test]
    fn test_pin_state_serialization() {
        let state = PinState::High;
        let serialized = serde_json::to_string(&state).unwrap();
        let deserialized: PinState = serde_json::from_str(&serialized).unwrap();
        assert_eq!(state, deserialized);
    }

    #[cfg(not(feature = "gpio"))]
    #[test]
    fn test_mock_gpio_provider() {
        let mut provider = MockGpio::new().unwrap();
        let status = provider.read_gpio_status().unwrap();
        assert!(!status.gpio_available);
        assert!(!provider.is_pin_available(18));
    }
}

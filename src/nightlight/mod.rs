// Nightlight/Gamma Control Component
// Inspired by Omarchy's nightlight toggle
// Provides screen temperature control for blue light filtering

/// Color temperature in Kelvin
#[derive(Debug, Clone, PartialEq)]
pub struct ColorTemperature {
    pub kelvin: u32,
}

impl ColorTemperature {
    pub fn new(kelvin: u32) -> Self {
        Self { kelvin }
    }

    /// Daylight temperature (6500K)
    pub fn daylight() -> Self {
        Self::new(6500)
    }

    /// Night temperature (4000K)
    pub fn night() -> Self {
        Self::new(4000)
    }

    /// Check if this is a warm (night) temperature
    pub fn is_warm(&self) -> bool {
        self.kelvin < 5000
    }

    /// Check if this is a cool (daylight) temperature
    pub fn is_cool(&self) -> bool {
        self.kelvin >= 5000
    }

    /// Get the opposite temperature
    pub fn opposite(&self) -> Self {
        if self.is_warm() {
            Self::daylight()
        } else {
            Self::night()
        }
    }
}

/// Nightlight schedule
#[derive(Debug, Clone)]
pub struct NightlightSchedule {
    pub enabled: bool,
    pub start_hour: u8,
    pub start_minute: u8,
    pub end_hour: u8,
    pub end_minute: u8,
    pub temperature: u32,
}

impl NightlightSchedule {
    pub fn new() -> Self {
        Self {
            enabled: false,
            start_hour: 20,
            start_minute: 0,
            end_hour: 7,
            end_minute: 0,
            temperature: 4000,
        }
    }

    pub fn with_time(
        mut self,
        start_hour: u8,
        start_minute: u8,
        end_hour: u8,
        end_minute: u8,
    ) -> Self {
        self.start_hour = start_hour;
        self.start_minute = start_minute;
        self.end_hour = end_hour;
        self.end_minute = end_minute;
        self
    }

    pub fn with_temperature(mut self, temp: u32) -> Self {
        self.temperature = temp;
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Validate schedule fields before applying them.
    pub fn validate(&self) -> Result<(), String> {
        if self.start_hour >= 24 || self.end_hour >= 24 {
            return Err("Schedule hours must be between 0 and 23".to_string());
        }
        if self.start_minute >= 60 || self.end_minute >= 60 {
            return Err("Schedule minutes must be between 0 and 59".to_string());
        }
        if !(1000..=10000).contains(&self.temperature) {
            return Err("Temperature must be between 1000K and 10000K".to_string());
        }
        Ok(())
    }

    /// Return whether a local wall-clock time falls in the schedule.
    /// The interval includes its start and excludes its end. Equal endpoints
    /// mean the schedule is active for the full day.
    pub fn is_active_at(&self, hour: u8, minute: u8) -> bool {
        if !self.enabled || hour >= 24 || minute >= 60 || self.validate().is_err() {
            return false;
        }

        let now = u16::from(hour) * 60 + u16::from(minute);
        let start = u16::from(self.start_hour) * 60 + u16::from(self.start_minute);
        let end = u16::from(self.end_hour) * 60 + u16::from(self.end_minute);
        if start == end {
            true
        } else if start < end {
            now >= start && now < end
        } else {
            now >= start || now < end
        }
    }
}

impl Default for NightlightSchedule {
    fn default() -> Self {
        Self::new()
    }
}

/// Nightlight manager
#[derive(Debug, Clone)]
pub struct NightlightManager {
    current_temperature: ColorTemperature,
    schedule: NightlightSchedule,
    is_enabled: bool,
}

impl NightlightManager {
    pub fn new() -> Self {
        Self {
            current_temperature: ColorTemperature::daylight(),
            schedule: NightlightSchedule::new(),
            is_enabled: false,
        }
    }

    /// Toggle nightlight on/off
    pub fn toggle(&mut self) -> ColorTemperature {
        if self.is_enabled {
            self.disable()
        } else {
            self.enable()
        }
    }

    /// Enable nightlight
    pub fn enable(&mut self) -> ColorTemperature {
        self.is_enabled = true;
        self.current_temperature = ColorTemperature::night();
        self.current_temperature.clone()
    }

    /// Disable nightlight
    pub fn disable(&mut self) -> ColorTemperature {
        self.is_enabled = false;
        self.current_temperature = ColorTemperature::daylight();
        self.current_temperature.clone()
    }

    /// Set custom temperature
    pub fn set_temperature(&mut self, kelvin: u32) -> Result<(), String> {
        if kelvin < 1000 || kelvin > 10000 {
            return Err("Temperature must be between 1000K and 10000K".to_string());
        }
        self.current_temperature = ColorTemperature::new(kelvin);
        Ok(())
    }

    /// Get current temperature
    pub fn get_temperature(&self) -> &ColorTemperature {
        &self.current_temperature
    }

    /// Check if nightlight is enabled
    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    /// Set schedule
    pub fn set_schedule(&mut self, schedule: NightlightSchedule) {
        self.schedule = schedule;
    }

    /// Get schedule
    pub fn get_schedule(&self) -> &NightlightSchedule {
        &self.schedule
    }

    /// Apply the configured schedule for the supplied local wall-clock time.
    pub fn apply_schedule_at(&mut self, hour: u8, minute: u8) -> Result<ColorTemperature, String> {
        self.schedule.validate()?;
        if hour >= 24 || minute >= 60 {
            return Err("Time must be between 00:00 and 23:59".to_string());
        }
        if self.schedule.is_active_at(hour, minute) {
            self.set_temperature(self.schedule.temperature)?;
            self.is_enabled = true;
            Ok(self.current_temperature.clone())
        } else {
            self.disable();
            Ok(self.current_temperature.clone())
        }
    }
}

impl Default for NightlightManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let manager = NightlightManager::new();
        assert!(!manager.is_enabled());
        assert_eq!(manager.get_temperature().kelvin, 6500);
    }

    #[test]
    fn test_toggle() {
        let mut manager = NightlightManager::new();
        let temp = manager.toggle();
        assert!(manager.is_enabled());
        assert_eq!(temp.kelvin, 4000);

        let temp = manager.toggle();
        assert!(!manager.is_enabled());
        assert_eq!(temp.kelvin, 6500);
    }

    #[test]
    fn test_enable_disable() {
        let mut manager = NightlightManager::new();

        manager.enable();
        assert!(manager.is_enabled());
        assert_eq!(manager.get_temperature().kelvin, 4000);

        manager.disable();
        assert!(!manager.is_enabled());
        assert_eq!(manager.get_temperature().kelvin, 6500);
    }

    #[test]
    fn test_set_temperature() {
        let mut manager = NightlightManager::new();

        let result = manager.set_temperature(5000);
        assert!(result.is_ok());
        assert_eq!(manager.get_temperature().kelvin, 5000);

        let result = manager.set_temperature(500);
        assert!(result.is_err());
    }

    #[test]
    fn test_color_temperature() {
        let warm = ColorTemperature::night();
        assert!(warm.is_warm());
        assert!(!warm.is_cool());

        let cool = ColorTemperature::daylight();
        assert!(!cool.is_warm());
        assert!(cool.is_cool());

        let opposite = warm.opposite();
        assert_eq!(opposite.kelvin, 6500);
    }

    #[test]
    fn test_schedule() {
        let schedule = NightlightSchedule::new()
            .with_enabled(true)
            .with_time(20, 0, 7, 0)
            .with_temperature(3500);

        assert!(schedule.enabled);
        assert_eq!(schedule.start_hour, 20);
        assert_eq!(schedule.temperature, 3500);
    }

    #[test]
    fn test_apply_schedule() {
        let mut manager = NightlightManager::new();
        let schedule = NightlightSchedule::new()
            .with_enabled(true)
            .with_time(20, 0, 7, 0)
            .with_temperature(3500);
        manager.set_schedule(schedule);

        let result = manager.apply_schedule_at(23, 0).unwrap();
        assert!(manager.is_enabled());
        assert_eq!(result.kelvin, 3500);

        let result = manager.apply_schedule_at(12, 0).unwrap();
        assert!(!manager.is_enabled());
        assert_eq!(result.kelvin, 6500);
    }
}

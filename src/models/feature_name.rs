use std::fmt;
use std::str::FromStr;

#[derive(Debug, thiserror::Error)]
#[error("invalid feature name")]
pub struct InvalidFeatureName;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureName {
    Temperature,
    Humidity,
    Light,
    Motion,
    AirQuality,
    AirPressure,
    Iaq,
    Voc,
    Eco2,
    Altitude,
    Co2,
    Level,
    Ph,
    Online,
    Mode,
}

impl FeatureName {
    pub const ALL: &[Self] = &[
        Self::Temperature,
        Self::Humidity,
        Self::Light,
        Self::Motion,
        Self::AirQuality,
        Self::AirPressure,
        Self::Iaq,
        Self::Voc,
        Self::Eco2,
        Self::Altitude,
        Self::Co2,
        Self::Level,
        Self::Ph,
        Self::Online,
        Self::Mode,
    ];

    pub fn is_float(self) -> bool {
        matches!(
            self,
            Self::Temperature
                | Self::Humidity
                | Self::Light
                | Self::AirPressure
                | Self::Mode
                | Self::Iaq
                | Self::Voc
                | Self::Eco2
                | Self::Altitude
                | Self::Ph
                | Self::Co2
        )
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Temperature => "temperature",
            Self::Humidity => "humidity",
            Self::Light => "light",
            Self::Motion => "motion",
            Self::AirQuality => "airquality",
            Self::AirPressure => "airpressure",
            Self::Iaq => "iaq",
            Self::Voc => "voc",
            Self::Eco2 => "eco2",
            Self::Altitude => "altitude",
            Self::Co2 => "co2",
            Self::Level => "level",
            Self::Ph => "ph",
            Self::Online => "online",
            Self::Mode => "mode",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FeatureName;

    #[test]
    fn mode_is_a_float_feature() {
        assert!(FeatureName::Mode.is_float());
    }
}

impl fmt::Display for FeatureName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for FeatureName {
    type Err = InvalidFeatureName;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "temperature" => Ok(Self::Temperature),
            "humidity" => Ok(Self::Humidity),
            "light" => Ok(Self::Light),
            "motion" => Ok(Self::Motion),
            "airquality" => Ok(Self::AirQuality),
            "airpressure" => Ok(Self::AirPressure),
            "iaq" => Ok(Self::Iaq),
            "voc" => Ok(Self::Voc),
            "eco2" => Ok(Self::Eco2),
            "altitude" => Ok(Self::Altitude),
            "co2" => Ok(Self::Co2),
            "level" => Ok(Self::Level),
            "ph" => Ok(Self::Ph),
            "online" => Ok(Self::Online),
            "mode" => Ok(Self::Mode),
            _ => Err(InvalidFeatureName),
        }
    }
}

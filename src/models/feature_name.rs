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
        Self::Online,
        Self::Mode,
    ];

    pub fn is_float(self) -> bool {
        matches!(self, Self::Temperature | Self::Humidity | Self::Light | Self::AirPressure)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Temperature => "temperature",
            Self::Humidity => "humidity",
            Self::Light => "light",
            Self::Motion => "motion",
            Self::AirQuality => "airquality",
            Self::AirPressure => "airpressure",
            Self::Online => "online",
            Self::Mode => "mode",
        }
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
            "online" => Ok(Self::Online),
            "mode" => Ok(Self::Mode),
            _ => Err(InvalidFeatureName),
        }
    }
}

use application::{ApplicationError, HapticPort};

#[derive(Clone, Default)]
pub struct AndroidHaptics;

impl AndroidHaptics {
    pub fn new() -> Self {
        Self
    }
}

impl HapticPort for AndroidHaptics {
    fn light_feedback(&self) -> Result<(), ApplicationError> {
        tracing::debug!("Haptic feedback: light");
        #[cfg(target_os = "android")]
        {
            // Android Vibrator.vibrate(VibrationEffect.createPredefined(EFFECT_CLICK))
        }
        Ok(())
    }

    fn success_feedback(&self) -> Result<(), ApplicationError> {
        tracing::debug!("Haptic feedback: success");
        #[cfg(target_os = "android")]
        {
            // Android Vibrator.vibrate(VibrationEffect.createPredefined(EFFECT_HEAVY_CLICK))
        }
        Ok(())
    }

    fn warning_feedback(&self) -> Result<(), ApplicationError> {
        tracing::debug!("Haptic feedback: warning");
        #[cfg(target_os = "android")]
        {
            // Android Vibrator.vibrate(VibrationEffect.createWaveform(...))
        }
        Ok(())
    }
}

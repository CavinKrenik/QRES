/// Calculates statistical correlations between data streams.
pub struct PearsonCorrelation;

impl PearsonCorrelation {
    /// Calculates the Pearson correlation coefficient between two signals.
    ///
    /// Returns a value between -1.0 (perfect inverse correlation) and 1.0 (perfect correlation).
    /// Returns 0.0 if arrays are different lengths or empty.
    ///
    /// Args:
    ///     x: First data stream.
    ///     y: Second data stream.
    ///
    /// Returns:
    ///     f32: The correlation coefficient.
    pub fn calculate(x: &[f32], y: &[f32]) -> f32 {
        if x.len() != y.len() || x.is_empty() {
            return 0.0;
        }

        let n = x.len() as f32;

        // Calculate means
        let mean_x = x.iter().sum::<f32>() / n;
        let mean_y = y.iter().sum::<f32>() / n;

        // Calculate covariance and variances using iterators
        let (covariance, var_x, var_y) =
            x.iter()
                .zip(y.iter())
                .fold((0.0, 0.0, 0.0), |(cov, vx, vy), (&xi, &yi)| {
                    let dx = xi - mean_x;
                    let dy = yi - mean_y;
                    (cov + dx * dy, vx + dx * dx, vy + dy * dy)
                });

        if var_x == 0.0 || var_y == 0.0 {
            return 0.0;
        }

        covariance / (var_x.sqrt() * var_y.sqrt())
    }
}

// SPDX-License-Identifier: MPL-2.0
/// Filter for the per-source routing key adapters use to demultiplex
/// dispatches from multiple sensors of the same type.
///
/// Centralised so all adapters share one definition of "matches" — if
/// glob/regex/prefix matching ever lands, it lands in one place.
///
/// # Example
///
/// ```
/// use isaac_sim_bridge::SourceFilter;
/// let f = SourceFilter::exact("/World/Carter/lidar_2d");
/// assert!(f.matches("/World/Carter/lidar_2d"));
/// assert!(!f.matches("/World/Carter/lidar_3d"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFilter(String);

impl SourceFilter {
    /// Construct a filter that matches exactly `source` (no wildcards).
    ///
    /// # Example
    ///
    /// ```
    /// use isaac_sim_bridge::SourceFilter;
    /// let f = SourceFilter::exact("/World/lidar");
    /// assert!(f.matches("/World/lidar"));
    /// ```
    pub fn exact(source: impl Into<String>) -> Self {
        Self(source.into())
    }

    /// Returns `true` if `src` matches this filter exactly.
    ///
    /// # Example
    ///
    /// ```
    /// use isaac_sim_bridge::SourceFilter;
    /// let f = SourceFilter::exact("/sensor/a");
    /// assert!(f.matches("/sensor/a"));
    /// assert!(!f.matches("/sensor/b"));
    /// ```
    pub fn matches(&self, src: &str) -> bool {
        self.0 == src
    }

    /// The raw filter string, e.g. a prim path like `/World/Carter/lidar_2d`.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for SourceFilter {
    fn from(s: String) -> Self {
        Self::exact(s)
    }
}

impl From<&str> for SourceFilter {
    fn from(s: &str) -> Self {
        Self::exact(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_exact_path() {
        let f = SourceFilter::exact("/Root/World/Carter/lidar_2d");
        assert!(f.matches("/Root/World/Carter/lidar_2d"));
        assert!(!f.matches("/Root/World/Carter/lidar_3d"));
        assert!(!f.matches(""));
    }

    #[test]
    fn empty_filter_does_not_match_unset_source() {
        let f = SourceFilter::exact("");
        assert!(f.matches(""));
        assert!(!f.matches("/anything"));
    }
}

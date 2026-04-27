/// Filter for the per-source routing key adapters use to demultiplex
/// dispatches from multiple sensors of the same type.
///
/// Centralised so all adapters share one definition of "matches" — if
/// glob/regex/prefix matching ever lands, it lands in one place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFilter(String);

impl SourceFilter {
    pub fn exact(source: impl Into<String>) -> Self {
        Self(source.into())
    }

    pub fn matches(&self, src: &str) -> bool {
        self.0 == src
    }

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

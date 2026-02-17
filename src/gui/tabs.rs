#![forbid(unsafe_code)]

#[cfg(feature = "gui")]
#[derive(Debug, Clone, Copy)]
pub enum Tab {
    Build,
    List,
    Extract,
    Verify,
}

#[cfg(feature = "gui")]
impl Default for Tab {
    fn default() -> Self {
        Self::Build
    }
}
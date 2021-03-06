#[derive(Debug, Clone)]
pub(crate) struct OpenOptionsExt {
    pub(crate) mode: u32,
    pub(crate) custom_flags: i32,
}

impl OpenOptionsExt {
    pub(crate) fn new() -> Self {
        Self {
            mode: 0o666,
            custom_flags: 0,
        }
    }
}

impl std::os::unix::fs::OpenOptionsExt for OpenOptionsExt {
    fn mode(&mut self, mode: u32) -> &mut Self {
        self.mode = mode;
        self
    }

    fn custom_flags(&mut self, flags: i32) -> &mut Self {
        self.custom_flags = flags;
        self
    }
}

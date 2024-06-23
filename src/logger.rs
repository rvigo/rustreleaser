use anyhow::Result;
use simple_logger::init_with_level;

// TODO include parameters to set the log level
pub fn init() -> Result<()> {
    init_with_level(log::Level::Info)?;

    Ok(())
}

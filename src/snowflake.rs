use std::time::Duration;

pub struct Snowflake(u64);

impl Snowflake {
    fn new() -> Self {
        let unix_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(
                |err| -> Result<std::time::Duration, std::time::SystemTimeError> {
                    Ok::<Duration, _>(err.duration())
                },
            )
            .unwrap()
            .as_millis();
        // Clear out
        let time_trunc = (unix_time << 16) as u64;
        Snowflake(time_trunc | (rand::random::<u16>() as u64))
    }
}
impl Into<u64> for Snowflake {
    fn into(self) -> u64 {
        self.0
    }
}

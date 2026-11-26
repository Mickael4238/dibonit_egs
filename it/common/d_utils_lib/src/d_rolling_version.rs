#[derive(Clone, Copy, Debug)]
pub enum RollingVersion {    // when receive :
    PreviousDeprecated = 3,  //  -> Raise an Error, the other app is too old, but we manage the case and send buffers with the PreviousDeprecated buffer version
    PreviousToDeploy   = 0,  //  -> Raise a warning, our app is newly deployed (but not the other), we manage the case and send PreviousToDeploy buffer version
    Current            = 1,  //  -> normal case, both apps are up to date; we can send Current buffer version
    UnknownNext        = 2,  //  -> receive UnknownNext ==> an app is sending a buffer version that is not yet known (her current is 1): should ask to send our current (0 - for her it will be PreviousToDeploy)
}
impl RollingVersion {
    pub const MASK: u8 = 0x03;
    pub const MAX: u8 = 3;
    /// Any updates to the D_buf_lib when in production should see a roll in the versions
    pub fn from_u8(item: u8) -> RollingVersion {
        match item & RollingVersion::MASK {
            0 => RollingVersion::PreviousToDeploy,
            1 => RollingVersion::Current,
            2 => RollingVersion::UnknownNext,
            3 => RollingVersion::PreviousDeprecated,
            _ => unreachable!(), // MASK ensures only 0-3 possible
        }
    }
}

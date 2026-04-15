use crate::{CbrApiClient, CbrApiError};

pub struct TspiClient<'a> {
    _parent: &'a CbrApiClient,
}

impl<'a> TspiClient<'a> {
    pub(crate) fn new(_parent: &'a CbrApiClient) -> Self {
        Self { _parent }
    }

    // Stub for TsPI
    pub async fn ping(&self) -> Result<bool, CbrApiError> {
        Ok(true)
    }
}

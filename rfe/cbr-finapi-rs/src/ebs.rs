use crate::{CbrApiClient, CbrApiError};

pub struct EbsClient<'a> {
    _parent: &'a CbrApiClient,
}

impl<'a> EbsClient<'a> {
    pub(crate) fn new(_parent: &'a CbrApiClient) -> Self {
        Self { _parent }
    }

    // Stub for EBS
    pub async fn ping(&self) -> Result<bool, CbrApiError> {
        Ok(true)
    }
}

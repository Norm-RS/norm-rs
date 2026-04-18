use crate::{
    client::{QueueTicket, SmevClient},
    SmevError,
};
use alloc::format;
use alloc::string::ToString;
use rfe_types::Inn;

fn extract_ticket(text: &str) -> Result<QueueTicket, SmevError> {
    let start = text
        .find("<TicketId>")
        .ok_or_else(|| SmevError::Payload("missing <TicketId> in SMEV response".into()))?
        + 10;
    let end = text
        .find("</TicketId>")
        .ok_or_else(|| SmevError::Payload("missing </TicketId> in SMEV response".into()))?;
    if end <= start {
        return Err(SmevError::Payload("malformed <TicketId> bounds".into()));
    }
    Ok(QueueTicket(text[start..end].to_string()))
}

pub struct FnsService<'a> {
    client: &'a SmevClient,
}

impl<'a> FnsService<'a> {
    pub fn new(client: &'a SmevClient) -> Self {
        Self { client }
    }

    /// Request INN validation and income check.
    /// Returns a QueueTicket because SMEV 4 is async.
    pub async fn check_inn_and_income(
        &self,
        inn: Inn,
        dob_dmy: &str,
    ) -> Result<QueueTicket, SmevError> {
        // Construct the SmevRequest payload (simplified)
        let payload = format!(
            r#"
            <SmevMessage>
                <Sender>BANKS</Sender>
                <Recipient>FNS</Recipient>
                <Payload>
                    <InnCheckReq>
                        <Inn>{}</Inn>
                        <Dob>{}</Dob>
                    </InnCheckReq>
                </Payload>
            </SmevMessage>
        "#,
            inn.as_str(),
            dob_dmy
        );

        let url = self.client.get_url("/api/v1/fns/check");
        let res = self
            .client
            .get_http()
            .post(&url)
            .header("Content-Type", "application/xml")
            .body(payload)
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(SmevError::Payload(format!(
                "SMEV Refused: {}",
                res.status()
            )));
        }

        let text = res.text().await?;
        extract_ticket(&text)
    }
}

pub struct FnsCheckResponse {
    pub is_valid: bool,
    pub income_confirmed: bool,
}

impl FnsCheckResponse {
    pub fn parse_xml(xml: &str) -> Result<Self, SmevError> {
        // We use quick-xml in reality, but mock it here for brevity.
        let is_valid = xml.contains("<IsValid>true</IsValid>");
        let income_confirmed = xml.contains("<IncomeConfirmed>true</IncomeConfirmed>");

        Ok(Self {
            is_valid,
            income_confirmed,
        })
    }
}

pub struct EsiaService<'a> {
    client: &'a SmevClient,
}

impl<'a> EsiaService<'a> {
    pub fn new(client: &'a SmevClient) -> Self {
        Self { client }
    }

    /// Request user profile from ЕСИА (Digital Profile 2026).
    pub async fn request_user_profile(&self, oid: &str) -> Result<QueueTicket, SmevError> {
        let payload = format!(
            r#"
            <SmevMessage>
                <Sender>BANKS</Sender>
                <Recipient>ESIA</Recipient>
                <Payload>
                    <EsiaProfileReq>
                        <Oid>{}</Oid>
                        <Scope>fullname birthdate passport</Scope>
                    </EsiaProfileReq>
                </Payload>
            </SmevMessage>
        "#,
            oid
        );

        let url = self.client.get_url("/api/v1/esia/profile");
        let res = self
            .client
            .get_http()
            .post(&url)
            .header("Content-Type", "application/xml")
            .body(payload)
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(SmevError::Payload(format!(
                "ESIA Refused: {}",
                res.status()
            )));
        }

        let text = res.text().await?;
        extract_ticket(&text)
    }
}

#[cfg(test)]
mod tests {
    use super::{extract_ticket, FnsCheckResponse};
    use alloc::format;

    #[test]
    fn extract_ticket_ok() {
        let t = extract_ticket("<Response><TicketId>abc</TicketId></Response>").unwrap();
        assert_eq!(t.0, "abc");
    }

    #[test]
    fn extract_ticket_missing_start() {
        let err = extract_ticket("<Response></Response>").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("missing <TicketId>"));
    }

    #[test]
    fn fns_check_response_parse_xml_ok() {
        let xml =
            "<Response><IsValid>true</IsValid><IncomeConfirmed>true</IncomeConfirmed></Response>";
        let parsed = FnsCheckResponse::parse_xml(xml).unwrap();
        assert!(parsed.is_valid);
        assert!(parsed.income_confirmed);
    }

    #[test]
    fn fns_check_response_parse_xml_negative() {
        let xml =
            "<Response><IsValid>false</IsValid><IncomeConfirmed>false</IncomeConfirmed></Response>";
        let parsed = FnsCheckResponse::parse_xml(xml).unwrap();
        assert!(!parsed.is_valid);
        assert!(!parsed.income_confirmed);
    }
}

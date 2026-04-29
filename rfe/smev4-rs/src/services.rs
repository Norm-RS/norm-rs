use crate::{
    client::{QueueTicket, SmevClient},
    SmevError,
};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::Reader;
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

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
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
            xml_escape(inn.as_str()),
            xml_escape(dob_dmy)
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

#[derive(Debug)]
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

    pub fn parse_xml_strict(xml: &str) -> Result<Self, SmevError> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();
        let mut is_valid = None;
        let mut income_confirmed = None;

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Start(e) if e.name() == QName(b"IsValid") => {
                    let v = reader.read_text(QName(b"IsValid"))?;
                    is_valid = Some(v.as_ref() == "true");
                }
                Event::Start(e) if e.name() == QName(b"IncomeConfirmed") => {
                    let v = reader.read_text(QName(b"IncomeConfirmed"))?;
                    income_confirmed = Some(v.as_ref() == "true");
                }
                Event::Eof => break,
                _ => {}
            }
            buf.clear();
        }

        let is_valid =
            is_valid.ok_or_else(|| SmevError::Payload("missing <IsValid> in XML".into()))?;
        let income_confirmed = income_confirmed
            .ok_or_else(|| SmevError::Payload("missing <IncomeConfirmed> in XML".into()))?;

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
            xml_escape(oid)
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

    #[test]
    fn fns_check_response_parse_xml_strict_ok() {
        let xml =
            "<Response><IsValid>true</IsValid><IncomeConfirmed>false</IncomeConfirmed></Response>";
        let parsed = FnsCheckResponse::parse_xml_strict(xml).unwrap();
        assert!(parsed.is_valid);
        assert!(!parsed.income_confirmed);
    }

    #[test]
    fn fns_check_response_parse_xml_strict_missing_field() {
        let xml = "<Response><IsValid>true</IsValid></Response>";
        let err = FnsCheckResponse::parse_xml_strict(xml).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("missing <IncomeConfirmed>"));
    }

    #[test]
    fn test_xml_injection_in_inn_is_escaped() {
        let escaped = super::xml_escape("7700<bad>&\"'");
        assert_eq!(escaped, "7700&lt;bad&gt;&amp;&quot;&apos;");
    }
}

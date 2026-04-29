use criterion::{black_box, criterion_group, criterion_main, Criterion};
use quick_xml::events::Event;
use quick_xml::Reader;
use smev4_rs::services::FnsCheckResponse;

fn generate_large_xml() -> String {
    let snippet = r#"
        <Record>
            <Inn>7700000000</Inn>
            <Status>Valid</Status>
            <Income>150000</Income>
        </Record>
    "#;
    let mut xml = String::from("<SmevResponse><Records>");
    for _ in 0..5000 {
        xml.push_str(snippet);
    }
    xml.push_str("</Records></SmevResponse>");
    xml
}

fn parse_record_count(xml: &str) -> usize {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut count = 0usize;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) if e.name().as_ref() == b"Record" => {
                count += 1;
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(_) => break,
        }
        buf.clear();
    }

    count
}

fn bench_xml_parsing(c: &mut Criterion) {
    let xml = generate_large_xml();

    c.bench_function("quick_xml_parse_1mb", |b| {
        b.iter(|| {
            let cnt = parse_record_count(black_box(&xml));
            black_box(cnt)
        })
    });

    let response =
        "<Response><IsValid>true</IsValid><IncomeConfirmed>true</IncomeConfirmed></Response>";
    c.bench_function("fns_parse_xml_strict", |b| {
        b.iter(|| {
            let parsed = FnsCheckResponse::parse_xml_strict(black_box(response)).unwrap();
            black_box(parsed.is_valid)
        })
    });
}

criterion_group!(benches, bench_xml_parsing);
criterion_main!(benches);

use std::io::Cursor;

use quick_xml::{
    Reader, Writer,
    events::{BytesEnd, Event},
};

use crate::jam_app::JamAppError;

pub(super) fn canonicalize_root(xml: &[u8], expected_root: &str) -> Result<Vec<u8>, JamAppError> {
    let mut reader = Reader::from_reader(Cursor::new(xml));
    let mut writer = Writer::new(Vec::with_capacity(xml.len()));
    let mut buffer = Vec::new();
    let mut depth = 0_usize;
    let mut saw_root = false;
    let mut root_closed = false;
    let mut source_root = None::<Vec<u8>>;
    loop {
        match reader.read_event_into(&mut buffer).map_err(xml_error)? {
            Event::Eof => break,
            Event::Start(start) if depth == 0 => {
                if saw_root
                    || root_closed
                    || !is_expected_or_dependency_root(start.name().as_ref(), expected_root)
                {
                    return Err(invalid("XML has more than one root element"));
                }
                let mut root = start.into_owned();
                source_root = Some(root.name().as_ref().to_vec());
                root.set_name(expected_root.as_bytes());
                writer.write_event(Event::Start(root)).map_err(xml_error)?;
                saw_root = true;
                depth = 1;
            }
            Event::Empty(empty) if depth == 0 => {
                if saw_root
                    || root_closed
                    || !is_expected_or_dependency_root(empty.name().as_ref(), expected_root)
                {
                    return Err(invalid(
                        "XML document root is not an expected serializer root",
                    ));
                }
                let mut root = empty.into_owned();
                root.set_name(expected_root.as_bytes());
                writer.write_event(Event::Empty(root)).map_err(xml_error)?;
                saw_root = true;
                root_closed = true;
            }
            Event::Start(start) => {
                writer
                    .write_event(Event::Start(start.into_owned()))
                    .map_err(xml_error)?;
                depth += 1;
            }
            Event::End(end) if depth == 1 => {
                if source_root.as_deref() != Some(end.name().as_ref()) {
                    return Err(invalid("XML root end does not match its start"));
                }
                writer
                    .write_event(Event::End(BytesEnd::new(expected_root)))
                    .map_err(xml_error)?;
                depth = 0;
                root_closed = true;
            }
            Event::End(end) => {
                if depth == 0 {
                    return Err(invalid("XML ended before its root started"));
                }
                writer
                    .write_event(Event::End(end.into_owned()))
                    .map_err(xml_error)?;
                depth -= 1;
            }
            event if depth == 0 => {
                reject_non_document_outside_root(&event)?;
                writer.write_event(event.into_owned()).map_err(xml_error)?;
            }
            event => writer.write_event(event.into_owned()).map_err(xml_error)?,
        }
        buffer.clear();
    }
    if !saw_root || !root_closed || depth != 0 {
        return Err(invalid("XML root is missing or unclosed"));
    }
    Ok(writer.into_inner())
}

pub(super) fn validate_canonical_root(xml: &[u8], expected_root: &str) -> Result<(), JamAppError> {
    let mut reader = Reader::from_reader(Cursor::new(xml));
    let mut buffer = Vec::new();
    let mut root_seen = false;
    let mut root_closed = false;
    let mut depth = 0_usize;
    loop {
        match reader.read_event_into(&mut buffer).map_err(xml_error)? {
            Event::Eof => break,
            Event::Start(start) => {
                if depth == 0 {
                    if root_seen || root_closed || start.name().as_ref() != expected_root.as_bytes()
                    {
                        return Err(invalid("XML document root is not canonical"));
                    }
                    root_seen = true;
                }
                depth += 1;
            }
            Event::End(end) => {
                if depth == 0 {
                    return Err(invalid("XML ended before its root started"));
                }
                if depth == 1 {
                    if end.name().as_ref() != expected_root.as_bytes() {
                        return Err(invalid("XML root end is not canonical"));
                    }
                    root_closed = true;
                }
                depth -= 1;
            }
            Event::Empty(empty) if depth == 0 => {
                if root_seen || root_closed || empty.name().as_ref() != expected_root.as_bytes() {
                    return Err(invalid("XML document root is not canonical"));
                }
                root_seen = true;
                root_closed = true;
            }
            event if depth == 0 => reject_non_document_outside_root(&event)?,
            _ => {}
        }
        buffer.clear();
    }
    if root_seen && root_closed && depth == 0 {
        Ok(())
    } else {
        Err(invalid("XML canonical root is missing or unclosed"))
    }
}

fn is_expected_or_dependency_root(name: &[u8], expected: &str) -> bool {
    name == expected.as_bytes() || name == format!("{expected}Type").as_bytes()
}

fn reject_non_document_outside_root(event: &Event<'_>) -> Result<(), JamAppError> {
    match event {
        Event::Decl(_) | Event::DocType(_) | Event::Comment(_) | Event::PI(_) => Ok(()),
        Event::Text(text) if (text.as_ref() as &[u8]).iter().all(u8::is_ascii_whitespace) => Ok(()),
        Event::Text(_) | Event::CData(_) | Event::GeneralRef(_) => {
            Err(invalid("XML has non-whitespace content outside its root"))
        }
        _ => Err(invalid("XML has invalid content outside its root")),
    }
}

fn invalid(message: &str) -> JamAppError {
    JamAppError::InvalidSession(message.into())
}
fn xml_error(error: impl std::fmt::Display) -> JamAppError {
    JamAppError::InvalidSession(format!("canonical DAW XML parse error: {error}"))
}

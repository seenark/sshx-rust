use std::borrow::Cow;
use std::sync::Arc;
use skim::prelude::*;
use crate::index::ConfigIndex;

struct HostItem {
    host: String,
    hostname: String,
    group: Option<String>,
    alias: Option<String>,
    description: Option<String>,
    port: Option<u16>,
}

impl SkimItem for HostItem {
    fn text(&self) -> Cow<'_, str> {
        let alias_str = self.alias.as_deref().unwrap_or("");
        Cow::Owned(format!(
            "{} {} {} {} {}",
            self.host,
            self.hostname,
            self.group.as_deref().unwrap_or(""),
            alias_str,
            self.description.as_deref().unwrap_or("")
        ))
    }

    fn display<'a>(&'a self, _context: DisplayContext<'a>) -> AnsiString<'a> {
        let alias_str = self
            .alias
            .as_deref()
            .map(|a| format!(" ({a})"))
            .unwrap_or_default();
        let desc_str = self
            .description
            .as_deref()
            .map(|d| format!(" — {d}"))
            .unwrap_or_default();
        let port_str = self.port.map(|p| format!(":{p}")).unwrap_or_default();
        let group_str = self.group.as_deref().map(|g| format!("[{g}] ")).unwrap_or_default();
        AnsiString::from(format!(
            "{}{}  {}{}{}{}",
            group_str, self.host, alias_str, self.hostname, port_str, desc_str
        ))
    }

    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        let mut preview = format!("Host: {}\nHostName: {}", self.host, self.hostname);
        if let Some(p) = self.port {
            preview.push_str(&format!("\nPort: {p}"));
        }
        if let Some(ref g) = self.group {
            preview.push_str(&format!("\nGroup: {g}"));
        }
        if let Some(ref a) = self.alias {
            preview.push_str(&format!("\nAlias: {a}"));
        }
        if let Some(ref d) = self.description {
            preview.push_str(&format!("\nDescription: {d}"));
        }
        ItemPreview::Text(preview)
    }
}

pub fn select_host(index: &ConfigIndex, prefilter: Option<&str>) -> Option<String> {
    let items: Vec<HostItem> = index
        .hosts
        .iter()
        .map(|h| HostItem {
            host: h.name.clone(),
            hostname: h.hostname.clone(),
            group: h.sshx.group.clone(),
            alias: h.sshx.alias.clone(),
            description: h.sshx.description.clone(),
            port: h.port,
        })
        .collect();

    let options = SkimOptionsBuilder::default()
        .height("50%".to_string())
        .multi(false)
        .preview(Some("".to_string()))
        .bind(vec!["Enter:accept".to_string()])
        .query(prefilter.map(|s| s.to_string()))
        .build()
        .expect("valid skim options");

    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();
    for item in items {
        let _ = tx.send(Arc::new(item));
    }
    drop(tx);

    let selected = Skim::run_with(&options, Some(rx));

    if selected.is_none() {
        return None;
    }

    let output = selected.unwrap();

    if output.is_abort {
        return None;
    }

    output
        .selected_items
        .first()
        .and_then(|item| item.as_any().downcast_ref::<HostItem>().map(|h| h.host.clone()))
}
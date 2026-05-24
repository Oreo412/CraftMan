use crate::gui::gui::GuiEvents;
use std::io;
use std::io::Write;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone)]
pub struct TuiWriter {
    tx: UnboundedSender<GuiEvents>,
}

impl TuiWriter {
    pub fn new(tx: UnboundedSender<GuiEvents>) -> Self {
        TuiWriter { tx }
    }
}

impl Write for TuiWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s = String::from_utf8_lossy(buf).to_string();

        let _ = self.tx.send(GuiEvents::AddStdoutLine(s));

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

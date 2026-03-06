#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EmailEnvelopeDetails<'a> {
    pub to: &'a str,
    pub subject: &'a str,
    pub body: &'a str,
}

impl<'a> EmailEnvelopeDetails<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_to(mut self, to: &'a str) -> Self {
        self.to = to;

        self
    }

    pub fn set_subject(mut self, subject: &'a str) -> Self {
        self.subject = subject;

        self
    }

    pub fn set_body(mut self, body: &'a str) -> Self {
        self.body = body;

        self
    }
}

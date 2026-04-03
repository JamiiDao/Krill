#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EmailEnvelopeDetails {
    pub to: String,
    pub subject: String,
    pub body: String,
}

impl EmailEnvelopeDetails {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_to(mut self, to: &str) -> Self {
        self.to = to.to_string();

        self
    }

    pub fn set_subject(mut self, subject: &str) -> Self {
        self.subject = subject.to_string();

        self
    }

    pub fn set_body(mut self, body: &str) -> Self {
        self.body = body.to_string();

        self
    }
}

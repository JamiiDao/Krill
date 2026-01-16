use bitcode::{Decode, Encode};

#[derive(Debug, PartialEq, Default, Eq, Clone, Copy, Encode, Decode)]
pub enum ConfigurationStateMachine {
    #[default]
    SetLanguage,
    SetColorScheme,
    SetOrganizationInfo,
    SetAdministrators,
    Configured,
}

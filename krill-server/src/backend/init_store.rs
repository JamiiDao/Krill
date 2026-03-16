use std::sync::OnceLock;

use async_dup::Arc;
use async_lock::RwLock;
use krill_common::{AdminConfiguration, KrillError, KrillResult, ServerConfigurationState};
use krill_mail::KrillSmtps;
use krill_store::KrillStorage;
use yansi::Paint;

pub static SERVER_KEY: OnceLock<[u8; 32]> = OnceLock::new();
pub static KRILL_STORAGE: OnceLock<KrillStorage> = OnceLock::new();
pub static SERVER_ORG_INFO: OnceLock<Vec<u8>> = OnceLock::new();
pub static SUPPORTED_LANGUAGES: OnceLock<Vec<String>> = OnceLock::new();
pub(crate) static ADMIN_SECRET: OnceLock<Arc<RwLock<AdminConfiguration>>> = OnceLock::new();
pub(crate) static SERVER_MAIL_CONNECTION: OnceLock<KrillSmtps> = OnceLock::new();

pub fn store() -> KrillResult<&'static KrillStorage> {
    KRILL_STORAGE
        .get()
        .ok_or(KrillError::GlobalStorageNotInitialized)
}

pub(crate) fn init_server_statics() -> KrillResult<()> {
    futures_lite::future::block_on(async {
        let store_init = KrillStorage::init().await?;

        KRILL_STORAGE
            .set(store_init)
            .or(Err(KrillError::GlobalStorageInitializeError))?;

        let store = store()?;

        let app_state = crate::backend::state::load_app_state(store).await?;
        load_server_key(store).await?;
        load_color_scheme(store).await?;
        load_supported_languages(store).await?;

        let cmd_print = ConfigPrint::new(100);

        if app_state == ServerConfigurationState::Uninitialized {
            use krill_common::AdminConfiguration;
            use yansi::Paint;

            let secret = AdminConfiguration::new();

            tracing::info!("NEW ADMIN SECRET CREATED");

            ADMIN_SECRET
                .set(Arc::new(RwLock::new(secret)))
                .expect("Admin secret already set !!!!!!!!!\n");

            cmd_print.print_header("APP IS NOT CONFIGURED");
            cmd_print.print_blank();
            cmd_print.calc_and_print_multiple(&[
                " ADMIN PASSCODE>  ".cyan().on_black(),
                (*ADMIN_SECRET
                    .get()
                    .expect("Admin secret not set!!!!")
                    .read()
                    .await
                    .secret()
                    .expect("Admin secret not generated!!!!")
                    .as_string_passcode()
                    .as_str())
                .red()
                .bold()
                .underline()
                .on_black(),
            ]);
            cmd_print.print_multiple_blanks(2);
            cmd_print.calc_and_print(
                "This is the 8 digit admininstrator passcode for the server."
                    .green()
                    .on_black(),
            );
            cmd_print.print_blank();
            cmd_print.calc_and_print_multiple(&[
                "This  admininstrator passcode is valid for "
                    .green()
                    .on_black(),
                "60 Minutes".red().bold().underline().on_black(),
            ]);
            cmd_print.print_blank();
            cmd_print.calc_and_print(
                "To generate a new one just restart the server."
                    .magenta()
                    .on_black(),
            );
            cmd_print.print_blank();
            cmd_print.calc_and_print_multiple(&[
                "In production run `".magenta().on_black(),
                "systemctl reload krill-server".red().bold().on_black(),
                "` to restart the server.".magenta().on_black(),
            ]);

            if !cfg!(debug_assertions) {
                cmd_print.print_multiple_blanks(4);
                cmd_print.calc_and_print(
                    "Press Enter to start Krill Server..."
                        .cyan()
                        .bold()
                        .underline()
                        .on_black(),
                );
            }
            cmd_print.print_multiple_blanks(2);
            cmd_print.print_header("END OF CONFIGURATION INFORMATION");

            if !cfg!(debug_assertions) {
                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .expect("Unable to read commandline input");
            }

            println!("\n\n\n");
        }

        Ok(())
    })
}

async fn load_server_key(store: &KrillStorage) -> KrillResult<()> {
    let secret = store.get_server_secret().await?;

    SERVER_KEY
        .set(secret)
        .or(Err(KrillError::UnableToSetServerSecret))
}

async fn load_color_scheme(store: &KrillStorage) -> KrillResult<()> {
    let scheme = store.get_org_info_bytes().await?;

    SERVER_ORG_INFO
        .set(scheme)
        .or(Err(KrillError::UnableToGetColorScheme))
}

async fn load_supported_languages(store: &KrillStorage) -> KrillResult<()> {
    let langs = store.get_supported_languages().await?;

    SUPPORTED_LANGUAGES
        .set(langs)
        .or(Err(KrillError::UnableToGetSupportedLanguages))
}

#[derive(Debug, Clone, Copy)]
pub struct ConfigPrint(usize);

impl ConfigPrint {
    pub fn new(span: usize) -> Self {
        Self(span)
    }

    pub fn print_blank(&self) {
        print!("{}", "+".yellow().bold().on_black());

        let blanks = self.0.checked_sub(2).expect("Painted blank is longer");

        (0..blanks).for_each(|_| {
            print!(" ");
        });

        println!("{}", "+".yellow().bold().on_black());
    }

    pub fn print_multiple_blanks(&self, times: u8) {
        (0..times).for_each(|_| {
            self.print_blank();
        })
    }

    pub fn calc_and_print_multiple(&self, painted_values: &[yansi::Painted<&str>]) {
        print!("{}", "+ ".yellow().bold().on_black());
        let mut count = 2usize;

        painted_values.iter().for_each(|painted_value| {
            print!("{painted_value}");

            count += painted_value.value.len();
        });

        let blank_count = self
            .0
            .checked_sub(count + 1)
            .expect("Painted value is longer");

        (0..blank_count).for_each(|_| {
            print!("{}", " ".yellow().bold().on_black());
        });

        println!("{}", "+".yellow().bold().on_black());
    }

    pub fn calc_and_print(&self, painted_value: yansi::Painted<&str>) {
        print!("{}", "+ ".yellow().bold().on_black());

        let mut count = 2usize;

        print!("{painted_value}");

        count += painted_value.value.len();

        let blank_count = self
            .0
            .checked_sub(count + 1)
            .expect("Painted value is longer");

        (0..blank_count).for_each(|_| {
            print!("{}", " ".yellow().bold().on_black());
        });

        println!("{}", "+".yellow().bold().on_black());
    }

    pub fn print_header(&self, info: &str) {
        let mut header = String::default();
        // `+4` here included the space before and after header info
        // and the `+ +`
        let count = info.len() + 4;

        let dashes = self.0.checked_sub(count).expect("Invalid header span") / 2;
        header.push('+');
        (0..dashes).for_each(|_| {
            header.push('-');
        });
        header.push(' ');
        if !count.is_multiple_of(2) {
            header.push(' ');
        }
        header.push_str(info);
        header.push(' ');
        (0..dashes).for_each(|_| {
            header.push('-');
        });
        header.push('+');

        println!("{}", header.yellow().bold().on_black());
    }
}

use krill_common::{KrillError, KrillResult, OrganizationInfo};

use crate::KrillStorage;

impl KrillStorage {
    pub const KEYSPACE_ORG_INFO: &str = "AllOrgDetails";

    pub const ORG_INFO_KEY: &str = "OrganizationInfo";
    pub const SMTPS_KEY: &str = "SMTPs";
    pub const ORG_DOMAIN_NAME: &str = "FQDN";
    pub const SOLANA_API_KEY_INFO: &str = "API_KEY_SOLANA";

    pub async fn set_all_org_details(
        &self,
        org_info: OrganizationInfo,
        smtps_uri: &str,
        api_key: &str,
        fqdn: &str,
    ) -> KrillResult<()> {
        let keyspace = self.org_info_keyspace();

        let org_info_exists = self.get_org_info().await?;

        if !org_info_exists.name.is_empty() {
            return Err(KrillError::Store("Organization Already exists".to_string()));
        }

        let mut kvs = Vec::<(String, Vec<u8>)>::default();
        kvs.push((Self::ORG_INFO_KEY.to_string(), bitcode::encode(&org_info)));
        kvs.push((Self::SMTPS_KEY.to_string(), bitcode::encode(&smtps_uri)));
        kvs.push((Self::ORG_DOMAIN_NAME.to_string(), bitcode::encode(&fqdn)));
        kvs.push((
            Self::SOLANA_API_KEY_INFO.to_string(),
            bitcode::encode(&api_key),
        ));

        self.set_op_many(keyspace, kvs).await
    }

    pub async fn set_org_info(&self, org_info: OrganizationInfo) -> KrillResult<()> {
        let keyspace = self.org_info_keyspace();

        self.set_op(keyspace, Self::ORG_INFO_KEY, org_info).await
    }

    pub async fn update_org_info(&self, org_info: OrganizationInfo) -> KrillResult<()> {
        let keyspace = self.org_info_keyspace();
        let org_info_exists = self.get_org_info().await?;

        if !org_info_exists.name.is_empty() {
            return Err(KrillError::Store("Organization Already exists".to_string()));
        }

        self.set_op(keyspace, Self::ORG_INFO_KEY, org_info).await
    }

    pub async fn get_org_info(&self) -> KrillResult<OrganizationInfo> {
        let bytes = self.get_org_info_bytes().await?;

        bitcode::decode(&bytes).or(Err(KrillError::UnableToDeserializeBrandingData))
    }

    pub async fn get_org_info_bytes(&self) -> KrillResult<Vec<u8>> {
        let keyspace = self.org_info_keyspace();

        match self.get_op(keyspace, Self::ORG_INFO_KEY).await? {
            Some(bytes) => Ok(bytes),
            None => {
                let info = OrganizationInfo::default();

                self.set_org_info(info.clone()).await?;

                Ok(bitcode::encode(&info))
            }
        }
    }

    pub async fn set_smtps_uri(&self, smtps_uri: &str) -> KrillResult<()> {
        let keyspace = self.org_info_keyspace();

        self.set_op(keyspace, Self::SMTPS_KEY, smtps_uri).await
    }

    pub async fn get_smtps_uri(&self) -> KrillResult<Option<String>> {
        let keyspace = self.org_info_keyspace();

        let bytes = self.get_op(keyspace, Self::SMTPS_KEY).await?;

        bytes
            .map(|bytes| bitcode::decode::<String>(&bytes).or(Err(KrillError::UnableToDecodeSmtps)))
            .transpose()
    }

    pub async fn set_solana_api_key(&self, api_key: &str) -> KrillResult<()> {
        let keyspace = self.org_info_keyspace();

        self.set_op(keyspace, Self::SOLANA_API_KEY_INFO, api_key)
            .await
    }

    pub async fn get_solana_api_key(&self) -> KrillResult<Option<String>> {
        let keyspace = self.org_info_keyspace();

        let bytes = self.get_op(keyspace, Self::SOLANA_API_KEY_INFO).await?;

        bytes
            .map(|bytes| {
                bitcode::decode::<String>(&bytes).or(Err(KrillError::UnableToDecodeSolanaApiKey))
            })
            .transpose()
    }

    pub async fn set_fqdn(&self, fqdn: &str) -> KrillResult<()> {
        let keyspace = self.org_info_keyspace();

        self.set_op(keyspace, Self::ORG_DOMAIN_NAME, fqdn).await
    }

    pub async fn get_fqdn(&self) -> KrillResult<Option<String>> {
        let keyspace = self.org_info_keyspace();

        let bytes = self.get_op(keyspace, Self::ORG_DOMAIN_NAME).await?;

        bytes
            .map(|bytes| {
                bitcode::decode::<String>(&bytes).or(Err(KrillError::UnableToOrgDomainName))
            })
            .transpose()
    }
}

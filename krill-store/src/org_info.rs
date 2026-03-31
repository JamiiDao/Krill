use krill_common::{KrillError, KrillResult, OrganizationInfo};

use crate::KrillStorage;

impl KrillStorage {
    pub const KEYSPACE_ORG_INFO: &str = "OrganizationInfo";

    pub async fn set_org_info(&self, org_info: OrganizationInfo) -> KrillResult<()> {
        let keyspace = self.org_info_keyspace();

        self.set_op(keyspace, Self::KEYSPACE_ORG_INFO, org_info)
            .await
    }

    pub async fn update_org_info(&self, org_info: OrganizationInfo) -> KrillResult<()> {
        let keyspace = self.org_info_keyspace();
        let org_info_exists = self.get_org_info().await?;

        if !org_info_exists.name.is_empty() {
            return Err(KrillError::Store("Organization Already exists".to_string()));
        }

        self.set_op(keyspace, Self::KEYSPACE_ORG_INFO, org_info)
            .await
    }

    pub async fn get_org_info(&self) -> KrillResult<OrganizationInfo> {
        let bytes = self.get_org_info_bytes().await?;

        bitcode::decode(&bytes).or(Err(KrillError::UnableToDeserializeBrandingData))
    }

    pub async fn get_org_info_bytes(&self) -> KrillResult<Vec<u8>> {
        let keyspace = self.org_info_keyspace();

        match self.get_op(keyspace, Self::KEYSPACE_ORG_INFO).await? {
            Some(bytes) => Ok(bytes),
            None => {
                let info = OrganizationInfo::default();

                self.set_org_info(info.clone()).await?;

                Ok(bitcode::encode(&info))
            }
        }
    }
}

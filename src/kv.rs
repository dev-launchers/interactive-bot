use cloudflare::framework::{
    async_api::{ApiClient, Client},
    auth::Credentials,
    endpoint::{Endpoint, Method},
    response::{ApiResponse, ApiResult},
    Environment, HttpApiClientConfig,
};
use failure::Fallible;

#[derive(Deserialize)]
pub struct KVConfig {
    token: String,
    account_id: String,
    namespace_id: String,
}

pub struct KVClient {
    client: Client,
    account_id: String,
    namespace_id: String,
}

impl KVClient {
    fn new(config: KVConfig) -> Fallible<KVClient> {
        let cred = Credentials::UserAuthToken {
            token: config.token,
        };
        let client = Client::new(
            cred,
            HttpApiClientConfig::default(),
            Environment::Production,
        )?;
        Ok(KVClient {
            client,
            account_id: config.account_id,
            namespace_id: config.namespace_id,
        })
    }

    async fn read(&self, key: String) -> ApiResponse<Guess> {
        self.client
            .request(&ReadKV {
                account_id: &self.account_id,
                namespace_id: &self.namespace_id,
                key: &key,
            })
            .await
    }

    async fn write(&self, key: String, val: Guess) -> ApiResponse<()> {
        self.client
            .request(&WriteKV {
                account_id: &self.account_id,
                namespace_id: &self.namespace_id,
                key: &key,
                val: val,
            })
            .await
    }
}

struct ReadKV<'a> {
    account_id: &'a str,
    namespace_id: &'a str,
    key: &'a str,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Guess {
    value: String,
    created_at: u64,
}

impl ApiResult for Guess {}

impl<'a> Endpoint<Guess, (), ()> for ReadKV<'a> {
    fn method(&self) -> Method {
        Method::Get
    }

    fn path(&self) -> String {
        format!(
            "accounts/{}/storage/kv/namespaces/{}/values/{}",
            self.account_id, self.namespace_id, self.key
        )
    }
}

struct WriteKV<'a> {
    account_id: &'a str,
    namespace_id: &'a str,
    key: &'a str,
    val: Guess,
}

impl<'a> Endpoint<(), (), Guess> for WriteKV<'a> {
    fn method(&self) -> Method {
        Method::Put
    }

    fn path(&self) -> String {
        format!(
            "accounts/{}/storage/kv/namespaces/{}/values/{}",
            self.account_id, self.namespace_id, self.key
        )
    }

    fn body(&self) -> Option<Guess> {
        Some(self.val.clone())
    }
}

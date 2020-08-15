use crate::models::FeatureFlag;

use mockall::automock;

pub type DB = ();
pub type DBConnection = Connection;
pub type SetOutput = Result<FeatureFlag, ()>;
pub type GetOutput = Result<FeatureFlag, ()>;

pub struct Backend {}

pub struct Connection {}

impl Connection {
    pub fn establish(_url: &str) -> Result<DBConnection, ()> {
        Ok(Self {})
    }
}

#[automock]
impl Backend {
    pub fn set(_conn: &DBConnection, _flag: FeatureFlag) -> SetOutput {
        Err(())
    }

    pub fn get(_conn: &DBConnection, _flag: FeatureFlag) -> GetOutput {
        Err(())
    }

    pub fn backend_name() -> &'static str {
        "null"
    }
}

// use mock::MockBackend;

// let mut mock = MockBackend::new();
// mock.expect_connection_name()
//     .returning(|| "xd");
// assert_eq!(5, mock::connection_name(4));
// lazy_static::lazy_static! {

//     let mock = MockBackend::default();

//     let ctx = MockBackend::backend_name_context();
//     ctx.expect().returning(|| "xd");

//     assert_eq!("topper", Backend::backend_name());

// }

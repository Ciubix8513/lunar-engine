use lunar_engine_derive::as_any;

use super::*;

struct TestAsset {
    id: Option<UUID>,
    initialized: bool,
    data: i32,
}
impl TestAsset {
    const fn new() -> Self {
        Self {
            id: None,
            initialized: false,
            data: 0,
        }
    }
}

impl Asset for TestAsset {
    #[as_any]
    fn get_id(&self) -> UUID {
        self.id.unwrap()
    }

    fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error + Send>> {
        self.initialized = true;
        self.data = 20;
        Ok(())
    }

    fn dispose(&mut self) {
        self.initialized = false;
        self.data = -20;
    }

    fn set_id(&mut self, id: UUID) -> Result<(), Error> {
        if self.id.is_some() {
            Err(Error::IdAlreadySet)
        } else {
            self.id = Some(id);
            Ok(())
        }
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }
}

#[test]
fn test_asset_registration() {
    let mut store = AssetStore::new();

    for _ in 0..100 {
        let a = TestAsset::new();
        store.register(a);
    }

    store.intialize_all().unwrap();
}

#[test]
fn test_asset_borrowing() {
    let mut store = AssetStore::new();

    let a = TestAsset::new();
    let id = store.register(a);
    store.intialize_all().unwrap();

    let a = store.get_by_id::<TestAsset>(id).unwrap();
    assert_eq!(a.borrow().data, 20);

    let a = store.get_by_type::<TestAsset>().unwrap();
    assert_eq!(a.borrow().data, 20);

    let a = TestAsset::new();
    let id = store.register(a);

    let a = store.get_by_id::<TestAsset>(id).unwrap();
    let mut borrow = a.borrow_mut();
    assert_eq!(borrow.data, 20);

    borrow.dispose();

    assert_eq!(borrow.data, -20);
}

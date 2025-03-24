use crate::{Config, Pallet};
use codec::{Decode, Encode};
use frame_support::{
    storage_alias, ensure,
    traits::{UncheckedOnRuntimeUpgrade, Get},
    pallet_prelude::Weight,
};

pub mod v0 {
    use super::*;

    #[storage_alias]
    pub type SomethingOld<T: Config> = StorageValue<Pallet<T>, u32>;

}

pub struct InnerMigrationV0ToV1<T: Config>(core::marker::PhantomData<T>);

impl<T: Config> UncheckedOnRuntimeUpgrade for InnerMigrationV0ToV1<T> {
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {

        assert!(v0::SomethingOld::<T>::exists(), "Old storage must exits");

        
        assert!(
            crate::Something::<T>::iter().next().is_none(),
            "New Storage must be empty before migration"
        );

        let old_value = v0::SomethingOld::<T>::get().ok_or("No value in old storage")?;
        Ok(old_value.encode())
    }

    fn on_runtime_upgrade() -> Weight {
        let mut weight = Weight::zero();
        if let Some(old_value) = v0::SomethingOld::<T>::take() {
            let account = T::AccountId::decode(&mut [1u8; 32].as_ref()).expect("Failed to decode into account ID");
            crate::Something::<T>::insert(&account, old_value);
            weight += T::DbWeight::get().reads_writes(1, 1);
        }
        weight
    }
        

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {

        ensure!(!v0::SomethingOld::<T>::exists(), "Old storage not removed");
        let old_value = u32::decode(&mut &state[..]).map_err(|_| sp_runtime::TryRuntimeError::from("failed to decode old value"))?;
        let account = T::AccountId::decode(&mut [1u8; 32].as_ref()).expect("Failed to decode into account ID");

        let new_structure_value = crate::Something::<T>::get(&account).ok_or("Migration failed value found")?;

        ensure!(new_structure_value == old_value, "Data Mismatch");

        ensure!(crate::Something::<T>::iter().count() == 1,
            "Extra entires detected"
        );
        Ok(())
    }
}

#[cfg(feature = "try-runtime")]
#[test]
fn test_storage_migration() {
    use crate::mock::{new_test_ext, Test};
    new_test_ext().execute_with(|| {
        v0::SomethingOld::<crate::mock::Test>::put(42);

        let state = InnerMigrationV0ToV1::<Test>::pre_upgrade().expect("Pre-upgrade should succeed");

        let weight = InnerMigrationV0ToV1::<Test>::on_runtime_upgrade();

        InnerMigrationV0ToV1::<Test>::post_upgrade(state).expect("Post-upgrade should succeed");
        let account = <Test as frame_system::Config>::AccountId::decode(&mut [1u8; 32].as_ref()).expect("Failed to decode into account ID");

        assert_eq!(
            crate::Something::<Test>::get(&account),
            Some(42),
        );
        assert!(
            !v0::SomethingOld::<Test>::exists(),
            "Old storage not cleared up"
        );
    })
}

pub type MigrateV0ToV1<T> = frame_support::migrations::VersionedMigration<
    0, // The migration will only execute when the on-chain storage version is 0
    1, // The on-chain storage version will be set to 1 after the migration is complete
    InnerMigrationV0ToV1<T>,
    crate::pallet::Pallet<T>,
    <T as frame_system::Config>::DbWeight,
>;

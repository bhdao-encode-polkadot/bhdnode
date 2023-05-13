use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_join_dao() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		assert_ok!(BhdaoModule::join_dao(RuntimeOrigin::signed(1),b"member1".to_vec()));
		assert_eq!(BhdaoModule::members_uid_count(), 1);
		// Read pallet storage and assert an expected result.
		//assert_eq!(BhdaoModule::something(), Some(42));
		// Assert that the correct event was deposited
		System::assert_last_event(Event::MemberAdded { who: 1, uid: 1 }.into());
	});
}

#[test]
fn it_fails_for_join_dao() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		assert_ok!(BhdaoModule::join_dao(RuntimeOrigin::signed(1),b"member1".to_vec()));
		assert_eq!(BhdaoModule::members_uid_count(), 1);
		
		assert_noop!(BhdaoModule::join_dao(RuntimeOrigin::signed(1),b"member1".to_vec()),Error::<Test>::MemberAlreadyExists);
	});
}


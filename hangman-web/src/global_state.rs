use crate::{storage, storage::StorageError};
use fermi::Atom;
use hangman_data::User;

pub static USER: Atom<Result<Option<User>, StorageError>> =
    |_| storage::load::<User>("hangman_user");

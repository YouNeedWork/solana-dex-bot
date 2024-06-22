use crate::user::User;

pub struct BuyAction<'a> {
    pub user: &'a User,
}

impl<'a> BuyAction<'a> {
    pub fn new(user: &'a User) -> Self { Self { user } }
}

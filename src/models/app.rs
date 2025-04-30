use crate::traits::auth::Auth;
use crate::traits::data_base::DataBase;
use crate::traits::start::Start;

pub struct App<C, D>
where C: Start,
    D: Start + Auth + DataBase,
{
    client: C,
    db: D,
}
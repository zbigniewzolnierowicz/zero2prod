use crate::domain::Email;
use crate::domain::SubscriberName;
use crate::routes::SubscribeFormBody;

pub struct NewSubscriber {
    pub name: SubscriberName,
    pub email: Email,
}

impl TryFrom<SubscribeFormBody> for NewSubscriber {
    type Error = String;

    fn try_from(value: SubscribeFormBody) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = Email::parse(value.email)?;

        Ok(Self { name, email })
    }
}

use crate::domain::SubscriberName;
use crate::domain::SubscriberEmail;
use crate::routes::SubscribeFormBody;

pub struct NewSubscriber {
    pub name: SubscriberName,
    pub email: SubscriberEmail,
}

impl TryFrom<SubscribeFormBody> for NewSubscriber {
    type Error = String;

    fn try_from(value: SubscribeFormBody) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;

        Ok(Self { name, email })
    }
}

use crate::entities;
use std::str::FromStr;
use ulid::Ulid;

#[derive(Debug, Clone)]
pub struct UlidScalar(pub Ulid);

#[juniper::graphql_scalar(name = "Ulid")]
impl<S> GraphQLScalar for UlidScalar
where
    S: juniper::ScalarValue,
{
    fn resolve(&self) -> juniper::Value {
        juniper::Value::scalar(self.0.to_string())
    }

    fn from_input_value(value: &juniper::InputValue) -> Option<UlidScalar> {
        value
            .as_string_value()
            .and_then(|s| Ulid::from_str(s).map(UlidScalar).ok())
    }

    fn from_str<'a>(value: juniper::ScalarToken<'a>) -> juniper::ParseScalarResult<'a, S> {
        <String as juniper::ParseScalarValue<S>>::from_str(value)
    }
}

#[derive(Debug, Clone)]
pub struct FigureScalar(pub entities::figure::Figure);

#[juniper::graphql_scalar(name = "Figure")]
impl<S> GraphQLScalar for FigureScalar
where
    S: juniper::ScalarValue,
{
    fn resolve(&self) -> juniper::Value {
        juniper::Value::scalar(self.0.to_json())
    }

    fn from_input_value(value: &juniper::InputValue) -> Option<FigureScalar> {
        value
            .as_string_value()
            .and_then(|s| entities::figure::Figure::from_json(s).map(FigureScalar))
    }

    fn from_str<'a>(value: juniper::ScalarToken<'a>) -> juniper::ParseScalarResult<'a, S> {
        <String as juniper::ParseScalarValue<S>>::from_str(value)
    }
}

#[derive(Debug, Clone)]
pub struct CharacterValueScalar(pub entities::character::Character);

#[juniper::graphql_scalar(name = "CharacterValue")]
impl<S> GraphQLScalar for CharacterValueScalar
where
    S: juniper::ScalarValue,
{
    fn resolve(&self) -> juniper::Value {
        juniper::Value::scalar(String::from(self.0.clone()))
    }

    fn from_input_value(value: &juniper::InputValue) -> Option<CharacterValueScalar> {
        value.as_string_value().and_then(|s| {
            entities::character::Character::try_from(s)
                .map(CharacterValueScalar)
                .ok()
        })
    }

    fn from_str<'a>(value: juniper::ScalarToken<'a>) -> juniper::ParseScalarResult<'a, S> {
        <String as juniper::ParseScalarValue<S>>::from_str(value)
    }
}

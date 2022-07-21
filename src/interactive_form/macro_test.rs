use crate::{interactive_form::InteractiveFormHooks, widgets::TextInputState};

#[crate::macros::interactive_form]
struct EmptyNamed {}

#[crate::macros::interactive_form]
struct EmptyUnamed();

#[crate::macros::interactive_form]
struct EmptyUnit;

struct Foo;
impl core::default::Default for Foo {
    fn default() -> Self {
        Self
    }
}

#[crate::macros::interactive_form]
struct OneFieldNamed {
    #[default("foo")]
    pub foo: TextInputState,
}

#[crate::macros::interactive_form]
struct OneFieldUnnamed(#[default("bar")] TextInputState);

#[test]
fn test_defaults() {
    EmptyUnit::default();
    EmptyNamed::default();
    EmptyUnamed::default();
    OneFieldNamed::default();
    OneFieldUnnamed::default();
}

#[test]
fn test_works() {
    let form = EmptyNamed::default();
    let focused = form.focused_state_idx();
    assert!(focused.is_none());
    assert_eq!(0, form.input_states_len());

    let form = OneFieldNamed::default();
    assert_eq!(1, form.input_states_len());
    assert_eq!("foo", form.foo.get_value());
}

use crate::{interactive_form::InteractiveFormHooks, widgets::TextInputState};

#[crate::macros::interactive_form]
struct Empty {}

struct Foo;
impl core::default::Default for Foo {
    fn default() -> Self {
        Self
    }
}

#[crate::macros::interactive_form]
struct WithOneField {
    #[default("foo")]
    pub foo: TextInputState,
}

#[test]
fn test_works() {
    let form = Empty::default();
    let focused = form.focused_state_idx();
    assert!(focused.is_none());
    assert_eq!(0, form.input_states_len());

    let form = WithOneField::default();
    assert_eq!(1, form.input_states_len());
}

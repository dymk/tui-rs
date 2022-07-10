///
/// Automatically create a backing struct and implementation for a form
/// of interactive input widgets.
///
/// Examples + doctests
///
/// => Private structs remain private
/// ```compile_fail
/// mod inner {
///     use tui::widgets::TextInputState;
///     tui::interactive_form_state!(struct TheForm {
///         bar: TextInputState
///     });
/// }
/// let form = inner::TheForm::default();
/// form.bar;
/// ```
///
/// => Pub structs work
/// ```
/// mod inner {
///     use tui::widgets::TextInputState;
///     tui::interactive_form_state!(pub struct TheForm {
///         bar: TextInputState
///     });
/// }
/// let form = inner::TheForm::default();
/// ```
///
/// => Private fields are private
/// ```compile_fail
/// mod inner {
///     use tui::widgets::TextInputState;
///     tui::interactive_form_state!(pub struct TheForm {
///         bar: TextInputState
///     });
/// }
/// let form = inner::TheForm::default();
/// form.bar;
/// ```
///
/// => Public fields are work
/// ```
/// mod inner {
///     use tui::widgets::TextInputState;
///     tui::interactive_form_state!(pub struct TheForm {
///         pub bar: TextInputState
///     });
/// }
/// let form = inner::TheForm::default();
/// assert_eq!("", form.bar.get_value());
/// ```
///
/// => Test empty structs generate properly
/// ```
/// tui::interactive_form_state!(struct TheForm {});
/// ```
///
/// => Test field defaults work
/// ```
/// use tui::widgets::TextInputState;
/// tui::interactive_form_state!(struct TheForm {
///     bar: TextInputState = "wow!"
/// });
/// let form = TheForm::default();
/// assert_eq!("wow!", form.bar.get_value());
/// ```
///
/// => Test trailing commas
/// ```
/// use tui::widgets::TextInputState;
/// tui::interactive_form_state!(struct TheForm {
///     bar: TextInputState,
/// });
/// ```
///
/// => Test trailing commas with default
/// ```
/// use tui::widgets::TextInputState;
/// tui::interactive_form_state!(struct TheForm {
///     bar: TextInputState = "wow!",
/// });
/// ```
///
#[macro_export]
macro_rules! interactive_form_state {
    // entrypoint
    ($struct_viz:vis struct $name:ident {
        $($field_viz:vis $field_name:ident : $field_type:ty $(= $default:expr)?),*
    }) => {
        $crate::interactive_form_state!(
            'make_struct_and_impl $struct_viz $name $($field_viz $field_name:$field_type $(= $default)?)*
        );
    };

    // entrypoint w/o trailing comma variant
    ($struct_viz:vis struct $name:ident {
        $($field_viz:vis $field_name:ident : $field_type:ty $(= $default:expr)?,)*
    }) => {
        $crate::interactive_form_state!(
            'make_struct_and_impl $struct_viz $name $($field_viz $field_name:$field_type $(= $default)?)*
        );
    };

    //
    // implementation details follow
    //

    // expands both `struct` and `impl`, passing in the needed repitition components
    (
        'make_struct_and_impl
        $struct_viz:vis
        $name:ident
        $($field_viz:vis $field_name:ident:$field_type:ty $(= $field_default:expr)?)*
    ) => {
        $crate::interactive_form_state!('make_struct $struct_viz $name $($field_viz $field_name:$field_type)*);
        $crate::interactive_form_state!('make_default $name
            [$([$field_name:$field_type$(=$field_default)?])*] []
        );
        $crate::interactive_form_state!('make_impl_form_hooks $name $($field_name)*);
    };

    // expand `struct` and its fields (with visibility modifiers)
    ('make_struct $struct_viz:vis $name:ident $($field_viz:vis $field_name:ident:$field_type:ty)*) => {
        $struct_viz struct $name {
            focused_state_idx: Option<usize>,
            $($field_viz $field_name: $field_type),*
        }

        impl $name {
            #[allow(dead_code)]
            pub fn input_states_len() -> usize {
                $crate::interactive_form_state!('count_idents $($field_name)*)
            }

            #[allow(dead_code)]
            #[allow(unused_variables)]
            pub fn input_state_at(&self, idx: usize) -> Option<&dyn $crate::widgets::InteractiveWidgetState> {
                $crate::interactive_form_state!(
                    'get_field_at_idx (&self) idx;
                    $($field_name)*
                )
            }
        }
    };

    // process field that has no default initializer - use Default::default()
    // instead
    ('make_default $name:ident
        [
            [$field_name:ident:$field_type:ty]
            $([$rest_name:ident:$rest_type:ty$(=$rest_default:expr)?])*
        ]
        [$([$processed_name:ident=$processed_default:expr])*]
    ) => {
        $crate::interactive_form_state!(
            'make_default $name
            [$([$rest_name:$rest_type$(=$rest_default)?])*]
            [$([$processed_name=$processed_default])* [$field_name=Default::default()]]
        );
    };

    // process field that has a default initializer - use `$field_expr.into()`
    // implement crate::widgets::ValueIntoWidgetState for types which can be
    // used as default initialzers for various concrete widget state types
    ('make_default $name:ident
        [
            [$field_name:ident:$field_type:ty=$field_default:expr]
            $([$rest_name:ident:$rest_type:ty$(=$rest_default:expr)?])*
        ]
        [$([$processed_name:ident=$processed_default:expr])*]
    ) => {
        $crate::interactive_form_state!(
            'make_default $name
            [$([$rest_name:$rest_type$(=$rest_default)?])*]
            [
                $([$processed_name=$processed_default])*
                [$field_name=$field_default.into()]
            ]
        );
    };

    // match on the case where all fields have been processed, and default
    // initializer expressions have been calculated - expand into a impl Default
    ('make_default $name:ident [] [$([$processed_name:ident=$processed_default:expr])*]) => {
        impl std::default::Default for $name {
            fn default() -> $name {
                $name {
                    focused_state_idx: None,
                    $(
                        $processed_name: $processed_default,
                    )*
                }
            }
        }
    };

    // expand `impl InteractiveFormHooks for $name`
    ('make_impl_form_hooks $name:ident $($field_name:ident)*) => {
        impl $crate::interactive_form::InteractiveFormHooks for $name {
            fn focused_state_idx(&self) -> Option<usize> {
                self.focused_state_idx
            }
            fn set_focused_state_idx(&mut self, idx: Option<usize>) {
                self.focused_state_idx = idx;
            }
            fn input_states_len(&self) -> usize {
                $name::input_states_len()
            }

            #[allow(unused_variables)]
            fn input_state_at_mut(&mut self, idx: usize) -> Option<&mut dyn $crate::widgets::InteractiveWidgetState> {
                $crate::interactive_form_state!(
                    'get_field_at_idx (&mut self) idx;
                    $($field_name)*
                )
            }
        }
    };

    // count_idents() => 0
    // count_idents(foo) => 1
    // count_idents(foo bar) => 2
    ('count_idents $($id:ident)*) => {
        <[()]>::len(&[$($crate::interactive_form_state!('count_idents $id ())),*])
    };
    ('count_idents $_t:ident $sub:expr) => {
        $sub
    };

    // get_field_at_idx((&self) idx;)
    //     => None

    // get_field_at_idx((&self) idx; foo)
    //     => if idx == 0 { Some(&self.foo) } else { None }

    // get_field_at_idx((&mut self) idx; foo)
    //     => if idx == 0 { Some(&mut self.foo) } else { None }

    // get_field_at_idx((&self) idx; foo bar)
    //     => if idx == 0 { Some(&mut &self.foo) } else
    //        if idx == 1 { Some(&mut &self.bar) } else { None }
    ('get_field_at_idx ($($self_ref:tt)*) $idx_id:ident; $($fields:ident)*) => {
        $crate::interactive_form_state!(
            'get_field_at_idx_impl
            ($($self_ref)*) $idx_id 0;
            $($fields)*
        )
    };

    // at least one field variant
    ('get_field_at_idx_impl ($($self_ref:tt)*) $idx_id:ident $idx:expr; $field:ident $($rest:ident)*) => {
        if $idx == $idx_id {
            Some($($self_ref)*.$field)
        } else {
            $crate::interactive_form_state!(
                'get_field_at_idx_impl
                ($($self_ref)*) $idx_id ($idx+1);
                $($rest)*
            )
        }
    };

    // no fields remain variant (None)
    ('get_field_at_idx_impl ($($self_ref:tt)*) $idx_id:ident $idx:expr; /* no fields */) => { None }
}

#[cfg(test)]
mod test {
    use crate::widgets::TextInputState;

    crate::interactive_form_state!(struct Foo {
        bar: TextInputState = "baz",
        quux: TextInputState = "smaz",
        whup: TextInputState
    });

    #[test]
    fn test_works() {
        let form = Foo::default();
        assert_eq!("baz", form.bar.get_value());
        assert_eq!("smaz", form.quux.get_value());
        assert_eq!("", form.whup.get_value());
    }
}

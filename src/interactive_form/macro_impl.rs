#[macro_export]
macro_rules! _tui_impl__count_idents {
    ($($id:ident)*) => {
        <[()]>::len(&[$($crate::_tui_impl__count_idents!(@inner $id ())),*])
    };
    (@inner $_t:ident $sub:expr) => {
        $sub
    };
}

#[macro_export]
macro_rules! _tui_impl__build_state_at_idx {
    ($self_id:ident; $idx_id:ident; $($id:ident)*) => {
        $crate::_tui_impl__build_state_at_idx![@inner $self_id; $idx_id; 0; $($id)*]
    };
    [@inner $self_id:ident; $idx_id:ident; $idx:expr; $id:ident $($rest:ident)*] => {
        if $idx == $idx_id {
            Some(&mut $self_id.$id)
        } else {
            $crate::_tui_impl__build_state_at_idx![@inner $self_id; $idx_id; ($idx+1); $($rest)*]
        }
    };
    [@inner $self_id:ident; $idx_id:ident; $idx:expr;] => { None }
}

#[macro_export]
macro_rules! interactive_form_state {
    ($name:ident { $($field_name:ident:$field_type:ty),* }) => {
        use $crate::interactive_form::InteractiveForm;

        #[derive(Default)]
        pub struct $name {
            focused_state_idx: Option<usize>,
            $(pub $field_name: $field_type),*
        }

        impl $crate::interactive_form::InteractiveFormHooks for $name {
            fn focused_state_idx(&self) -> Option<usize> {
                self.focused_state_idx
            }
            fn set_focused_state_idx(&mut self, idx: Option<usize>) {
                self.focused_state_idx = idx;
            }
            fn input_states_len(&self) -> usize {
                $crate::_tui_impl__count_idents!($($field_name)*)
            }
            fn input_state_at_mut(&mut self, idx: usize) -> Option<&mut dyn $crate::widgets::InteractiveWidgetState> {
                $crate::_tui_impl__build_state_at_idx!(self; idx; $($field_name)*)
            }
        }
    }
}

#[cfg(test)]
mod test_macro {
    use crate::widgets::TextInputState;

    // trace_macros!(true);
    interactive_form_state!(MyInteractiveForm {
        foo: TextInputState,
        bar: TextInputState
    });
    // trace_macros!(false);

    #[test]
    fn test_works() {
        println!();
    }
}

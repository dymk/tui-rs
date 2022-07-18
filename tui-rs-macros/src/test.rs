struct MyForm {
    _focused_state_idx: Option<usize>,
    #[default("FooDefault")]
    foo: TextInputState,
    bar: TextInputState
}
impl ::core::default::Default for MyForm {
    fn default() -> Self {
        Self {
            foo: ("FooDefault").into(),
            bar: ::core::default::Default::default(),
        }
    }
}
impl ::tui::interactive_form::InteractiveFormHooks for MyForm {
    fn focused_state_idx(&self) -> Option<usize> {
        self._focused_state_idx
    }
    fn set_focused_state_idx(&mut self, idx: Option<usize>) {
        self._focused_state_idx = idx;
    }
    fn input_states_len(&self) -> usize {
        2usize
    }
    fn input_state_at(&self, idx: usize) -> Option<&dyn::tui::widgets::InteractiveWidgetState> {
        todo!()
    }
    fn input_state_at_mut(
        &mut self,
        idx: usize,
    ) -> Option<&mut dyn::tui::widgets::InteractiveWidgetState> {
        todo!()
    }
}

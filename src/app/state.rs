use autolink_lib::Plan;
/*
 * since egui renders in immediate mode, the ui has to keep its own state
 * this struct is where those variables will live
 * 
 * first of all, the combobox which determines which 
 * section is drawn 
*/
pub struct UIState {
    pub section: usize,
    pub prev_section: usize,
    pub start: StartUIState,
    pub add: AddUIState,
    pub edit: EditUIState,
    pub remove: RemoveUIState,
}

impl Default for UIState {
    fn default() -> Self {
        Self {
            section: 0,
            prev_section: 0,
            start: StartUIState::default(),
            add: AddUIState::default(),
            edit: EditUIState::default(),
            remove: RemoveUIState::default(),
        }
    }
}

impl UIState {
    pub fn set_sections_to_default(&mut self) {
        self.start = StartUIState::default();
        self.add = AddUIState::default();
        self.edit = EditUIState::default();
        self.remove = RemoveUIState::default();
    }
}

pub struct StartUIState {
    pub selection: usize,
}

impl Default for StartUIState {
    fn default() -> Self {
        Self {
            selection: 0,
        }
    }
}

pub struct AddUIState {
    pub name: String,
    pub link: String,
    pub add_time: bool,
    pub selected_day: usize,
    pub hour: usize,
    pub minute: usize,
    pub output: String,
}

impl Default for AddUIState {
    fn default() -> Self {
        Self {
            name: String::new(),
            link: String::new(),
            add_time: false,
            selected_day: 0,
            hour: 0,
            minute: 0,
            output: String::new(),
        }
    }
}

pub struct EditUIState {
    pub selection: usize,
    pub prev_selection: usize,
    pub plan: Plan,
    pub add_time: bool,
    pub selected_day: usize,
    pub hour: usize,
    pub minute: usize,
    pub remove_time: bool,
    pub selected_time: usize,
    pub output: String,
}

impl EditUIState {
    pub fn refresh(&mut self, r: std::ops::Range<usize>) {
        let mut new = Self::default();
        new.selection = self.selection;
        new.prev_selection = Self::prev_selection_generate(r, self.selection);
        *self = new;
    }

    fn prev_selection_generate(r: std::ops::Range<usize>, not: usize) -> usize {
        let mut prev_sel = 0;
        for i in r {
            if i != not {
                prev_sel = i;
                break
            } 
        }
        prev_sel
    }
}



impl Default for EditUIState {
    fn default() -> Self {
        Self {
            selection: 0,
            prev_selection: 1, // this is for the update() function to assign a valid plan to the "plan" field
            plan: Plan { name: String::new(), link: String::new(), times: vec![] },
            add_time: false,
            selected_day: 0,
            hour: 0,
            minute: 0,
            remove_time: false,
            selected_time: 0,
            output: String::new(),
        }
    }
}

pub struct RemoveUIState {
    pub selection: usize,
    pub output: String,
}

impl Default for RemoveUIState {
    fn default() -> Self {
        Self {
            selection: 0,
            output: String::new(),
        }
    }
}

/*
 * WRITE CHECKS TO PREVENT CRASHES IF "PLANS" IS EMPTY
 * WRITE THE REMOVE UI AND LOGIC
*/
use iced::Task;

pub struct AlegriaAction<I, Message> {
    pub instructions: Vec<I>,
    pub tasks: Vec<Task<Message>>,
}

impl<I, Message> AlegriaAction<I, Message> {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            tasks: Vec::new(),
        }
    }

    pub fn add_instruction(&mut self, instruction: I) {
        self.instructions.push(instruction);
    }

    pub fn add_task(&mut self, task: Task<Message>) {
        self.tasks.push(task);
    }
}

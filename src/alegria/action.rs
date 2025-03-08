// SPDX-License-Identifier: GPL-3.0-only

use iced::Task;

pub struct AlegriaAction<I, Message>
where
    I: std::fmt::Debug + Clone,
{
    pub instructions: Vec<I>,
    pub tasks: Vec<Task<Message>>,
}

impl<I, Message> AlegriaAction<I, Message>
where
    I: std::fmt::Debug + Clone,
{
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

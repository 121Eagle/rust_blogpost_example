use std::cell::RefCell;
use std::rc::Rc;

pub struct Post {
    state: Option<Box<dyn State>>,
    content: Rc<RefCell<String>>,
}

impl Post {
    pub fn new() -> Post {
        let content = Rc::new(RefCell::new(String::new()));
        Post {
            state: Some(Box::new(Draft {content: content.clone()})),
            content,
        }
    }

    pub fn add_text(&mut self, text: &str) {
        self.content.borrow_mut().push_str(text);
    }

    pub fn content(&self) -> &str {
        self.state.as_ref().unwrap().content(self)
    }

    pub fn request_review(&mut self) {
        if let Some(s) = self.state.take() {
            self.state = Some(s.request_review())
        }
    }

    pub fn approve(&mut self) {
        if let Some(s) = self.state.take() {
            self.state = Some(s.approve())
        }
    }

    pub fn reject(&mut self) {
        if let Some(s) = self.state.take() {
            self.state = Some(s.reject())
        }
    }
}

impl Default for Post {
    fn default() -> Self {
        Self::new()
    }
}

trait State {
    fn request_review(self: Box<Self>) -> Box<dyn State>;
    fn approve(self: Box<Self>) -> Box<dyn State>;

    fn content<'a>(&self, post: &'a Post) -> &'a str {
        ""
    }

    fn reject(self: Box<Self>) -> Box<dyn State>;

    fn edit_text<'a>(&self, post_content: &'a mut String) -> Result<&'a mut String, &'static str> {
        Err("Cannot edit text unless state is draft")
    }
}

struct Draft {
    content: Rc<RefCell<String>>
}

impl State for Draft {
    fn request_review(self: Box<Self>) -> Box<dyn State> {
        Box::new(PendingReview { approvals: 0u8, content: self.content.clone() })
    }

    fn approve(self: Box<Self>) -> Box<dyn State> {
        self
    }

    fn reject(self: Box<Self>) -> Box<dyn State> {
        self
    }

    fn edit_text<'a>(&self, post_content: &'a mut String) -> Result<&'a mut String, &'static str> {
        Ok(post_content)
    }
}

struct PendingReview {
    approvals: u8,
    content: Rc<RefCell<String>>
}

impl State for PendingReview {
    fn request_review(self: Box<Self>) -> Box<dyn State> {
        self
    }

    fn approve(self: Box<Self>) -> Box<dyn State> {
        const NEEDED_APPROVALS: u8 = 2u8;

        let approval_count = self.approvals;
        if approval_count < NEEDED_APPROVALS {
            Box::new(Self {
                approvals: approval_count + 1u8,
                content: self.content.clone(),
            })
        } else {
            Box::new(Published {})
        }
    }

    fn reject(self: Box<Self>) -> Box<dyn State> {
        Box::new(Draft {content: self.content.clone()})
    }
}

struct Published {}

impl State for Published {
    fn request_review(self: Box<Self>) -> Box<dyn State> {
        self
    }

    fn approve(self: Box<Self>) -> Box<dyn State> {
        self
    }

    fn content<'a>(&self, post: &'a Post) -> &'a str {
        &*post.content()
    }

    fn reject(self: Box<Self>) -> Box<dyn State> {
        self
    }
}

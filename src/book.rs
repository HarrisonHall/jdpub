use super::*;

pub struct Book {
    pub title: String,
    pub author: String,
    pub chapters: Vec<Chapter>,
}

pub struct Chapter {
    pub title: Option<String>,
    pub ast: durf::Ast,
    // pub html: html::
    // pub html: crate:
}

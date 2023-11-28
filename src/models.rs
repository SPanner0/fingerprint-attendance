use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
/// To handle adding a new student
pub struct NewStudent {
    pub name: String,
    pub student_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
/// Represent a student from a JSON request body or a database
pub struct Student {
    pub name: String,
    pub student_id: String,
    pub fingerprint_id: i64,
    pub attendance: bool,
}

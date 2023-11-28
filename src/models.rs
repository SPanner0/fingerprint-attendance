use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct NewStudent {
    pub name: String,
    pub student_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Student {
    pub name: String,
    pub student_id: String,
    pub fingerprint_id: i64,
    pub attendance: bool,
}

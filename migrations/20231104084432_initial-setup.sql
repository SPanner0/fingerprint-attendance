-- Add migration script here

CREATE TABLE IF NOT EXISTS student (
    name VARCHAR(255) NOT NULL,
    student_id VARCHAR(255) UNIQUE NOT NULL,
    fingerprint_id INT NOT NULL,
    attendance BOOLEAN NOT NULL CHECK(attendance IN (0,1))
);